use chrono::{DateTime, NaiveDate, Datelike, Utc};
use reqwest::StatusCode;
use chrono_tz::Tz;

use super::day;
use crate::time::Time;
use crate::api::WebResponse;
use crate::sunrise_api::APIResponseDay;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Timer {
    /// includes leap day
    #[serde(with = "serde_big_array::BigArray")]
    day_timers: [day::Timer; 366]
}

/// intermediary representation for calculations
#[derive(PartialEq)]
struct LocalDay {
    length: Time,
    /// exactly in between sunrise and sunset
    center: Time,
}

impl Timer {
    /// day timers include leap day
    #[allow(clippy::large_types_passed_by_value)]
    pub const fn new(day_timers: [day::Timer; 366]) -> Self {
        Self { day_timers }
    }

    pub const fn day_timers(&self) -> &[day::Timer; 366] {
        &self.day_timers
    }

    pub fn for_today(&self, timezone: Tz) -> &day::Timer {
        let now = Utc::now().with_timezone(&timezone);

        &self.day_timers[Self::index(now)]
    }

    /// if `Ok`, returns tuple of
    /// - local timezone
    /// - actual year timer (given `natural_factor`)
    /// - local year timer (`natural_factor == 0.0`)
    /// - natural year timer (`natural_factor == 1.0`)
    pub fn from_api_days_average(natural_factor: f32, local_api_days: &[APIResponseDay], natural_api_days: &[APIResponseDay])
        -> WebResponse<(Tz, Self, Self, Self)>
    {
        assert!(natural_factor >= 0.);
        assert!(natural_factor <= 1.);
        assert_eq!(local_api_days.len(), 366);
        assert_eq!(natural_api_days.len(), 366);

        let timezone = Self::map_api_day_field(&local_api_days[0].timezone)?;
        let timezone = Time::zone_from(timezone);
        log::info!("using timezone {timezone}, current time is {}", Time::now(timezone));

        let local_days = local_api_days.iter()
            .map(|local_item| -> WebResponse<LocalDay> {
                let day_length = Self::map_api_day_field(&local_item.day_length)?;
                let length = Time::from_hhmmss(&day_length);
                let center = {
                    let sunrise = Self::map_api_day_field(&local_item.sunrise)?;
                    let sunrise = Time::from_military(sunrise);
                    let sunset = Self::map_api_day_field(&local_item.sunset)?;
                    let sunset = Time::from_military(sunset);
                    ((sunset - sunrise) / 2.0) + sunrise
                };
                Ok(LocalDay { length, center })
            }).collect::<Vec<_>>();

        // return the first error if present
        if let Some(Err(error)) = local_days.iter().find(|r| r.is_err()) {
            return Err(error.clone());
        }

        let local_days = local_days.into_iter()
            .map(|result| result.unwrap())
            .collect::<Vec<_>>();

        let natural_day_lengths = natural_api_days.iter()
            .map(|natural_item| -> WebResponse<Time> {
                let day_length = Self::map_api_day_field(&natural_item.day_length)?;
                Ok(Time::from_hhmmss(day_length))
            }).collect::<Vec<_>>();

        // return the first error if present
        if let Some(Err(error)) = natural_day_lengths.iter().find(|r| r.is_err()) {
            return Err(error.clone());
        }

        let mut natural_day_lengths = natural_day_lengths.into_iter()
            .map(|result| result.unwrap())
            .collect::<Vec<_>>();

        let local_max = local_days.iter()
            .max_by(|a, b| a.length.cmp(&b.length))
            .unwrap();
        let local_max_index = local_days.iter().position(|d| d == local_max).unwrap();

        let natural_max = natural_day_lengths.iter().max().unwrap();
        let natural_max_index = natural_day_lengths.iter().position(|t| t == natural_max).unwrap();

        // shift natural day lengths to ensure
        // longest natural day is at the date of the longest local day.
        // this is especially useful if local and natural location are in different hemispheres.
        // value is between -365 and 365.
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        let shift = local_max_index as i16 - natural_max_index as i16;
        log::debug!("longest day: local = {local_max_index}, natural = {natural_max_index} => shift natural by {shift}");

        if shift >= 0 {
            natural_day_lengths.rotate_right(shift.try_into().unwrap());
        } else {
            natural_day_lengths.rotate_left(shift.abs().try_into().unwrap());
        }

        // skip averaging if possible
        let year_timer = if natural_factor == 0. {
            Self::from_api_days(local_api_days)?
        } else {
            Self::average(&local_days, &natural_day_lengths, natural_factor)
        };

        let local_year_timer = if natural_factor == 0. {
            year_timer
        } else {
            Self::average(&local_days, &natural_day_lengths, 0.)
        };

        let natural_year_timer = if (natural_factor - 1.).abs() < f32::EPSILON {
            year_timer
        } else {
            Self::average(&local_days, &natural_day_lengths, 1.)
        };

        Ok((timezone, year_timer, local_year_timer, natural_year_timer))
    }

    /// compute year timer using a `natural_factor`
    fn average(local_days: &[LocalDay], natural_day_lengths: &[Time], natural_factor: f32) -> Self {
        assert_eq!(local_days.len(), 366);
        assert_eq!(natural_day_lengths.len(), 366);

        let day_timers = local_days.iter()
            .zip(natural_day_lengths.iter())
            .map(|(local_day, natural_day_length)| {
                let averaged_day_length = (*natural_day_length * natural_factor)
                    + (local_day.length * (1. - natural_factor));
                let on  = local_day.center - (averaged_day_length / 2.);
                let off = local_day.center + (averaged_day_length / 2.);
                day::Timer::new(on, off)
            })
            .collect::<Vec<_>>();
            
        Self::new(day_timers.try_into().unwrap())
    }

    fn from_api_days(api_days: &[APIResponseDay]) -> WebResponse<Self> {
        assert_eq!(api_days.len(), 366);

        let day_timers = api_days.iter()
            .map(|day| -> WebResponse<day::Timer> {
                let sunrise = Self::map_api_day_field(&day.sunrise)?;
                let sunset = Self::map_api_day_field(&day.sunset)?;
                Ok(day::Timer::new(
                    Time::from_military(&sunrise),
                    Time::from_military(&sunset),
                ))
            }).collect::<Vec<_>>();

        // return the first error if present
        if let Some(Err(error)) = day_timers.iter().find(|r| r.is_err()) {
            return Err(error.clone());
        }

        let day_timers = day_timers.into_iter()
            .map(|result| result.unwrap())
            .collect::<Vec<_>>();

        Ok(Self::new(day_timers.try_into().unwrap()))
    }

    /// returns index of day timers to use for given moment in time
    fn index(now: DateTime<Tz>) -> usize {
        let leap_year = now.date_naive().leap_year();
        let day = now.ordinal0();

        let leap_day_index = NaiveDate::from_ymd_opt(2000, 2, 29).unwrap().ordinal0();
        let mut index = day;

        // skip leap day entry if it's not a leap year
        if !leap_year && index >= leap_day_index {
            index += 1;
        } 

        index.try_into().unwrap()
    }

    /// map a field of an `APIResponseDay` to a `WebResponse`
    fn map_api_day_field<T>(option: &Option<T>) -> WebResponse<&T> {
        match option {
            Some(t) => Ok(t),
            None => Err((StatusCode::BAD_GATEWAY, String::from("Error while processing sunrise API response, a required field was null"))),
        }
    }
}

impl std::fmt::Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0 .. self.day_timers.len() {
            writeln!(f, "{:3} {}", i + 1, self.day_timers[i])?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index_test(year: i32, month: u32, day: u32, index: usize) {
        use chrono::TimeZone;
        let time = chrono_tz::CET.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap();
        assert_eq!(Timer::index(time), index);
    }

    #[test]
    fn index_zero() {
        index_test(2000, 1, 1, 0);
    } 

    #[test]
    fn index_max() {
        index_test(2000, 12, 31, 365); // leap year
        index_test(2001, 12, 31, 365); // normal year
    } 

    #[test]
    fn index_leap_day() {
        // leap year
        index_test(2000, 2, 28, 58);
        index_test(2000, 2, 29, 59);
        index_test(2000, 3,  1, 60);

        // normal year
        index_test(2001, 2, 28, 58);
        //index_test(2001, 2, 29, 59); // => panic, not a leap year
        index_test(2001, 3,  1, 60);
    }
}
