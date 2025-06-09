use chrono::{DateTime, NaiveDate, Datelike, Utc};
use chrono_tz::Tz;

use super::day;
use crate::time::Time;
use crate::sunrise_api::APIResponseDay;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Timer {
    timezone: Tz,
    /// includes leap day
    #[serde(with = "serde_big_array::BigArray")]
    day_timers: [day::Timer; 366]
}

impl Timer {
    /// day timers include leap day
    #[allow(clippy::large_types_passed_by_value)]
    pub const fn new(timezone: Tz, day_timers: [day::Timer; 366]) -> Self {
        Self { timezone, day_timers }
    }

    pub fn for_today(&self) -> &day::Timer {
        let now = Utc::now().with_timezone(&self.timezone);

        &self.day_timers[Self::index(now)]
    }

    pub fn from_api_days(api_days: &[APIResponseDay], timezone: Tz) -> Self {
        assert_eq!(api_days.len(), 366);

        Self::new(
            timezone,
            api_days.iter().map(|day| {
                day::Timer::new(
                    Time::from_military(&day.sunrise),
                    Time::from_military(&day.sunset)
                )
            }).collect::<Vec<_>>().try_into().unwrap()
        )
    }

    pub fn from_api_days_average(natural_factor: f32, local_api_days: &[APIResponseDay], natural_api_days: &[APIResponseDay]) -> Self {
        #[derive(PartialEq)]
        struct LocalDay {
            length: Time,
            /// exactly in between sunrise and sunset
            center: Time,
        }

        assert!(natural_factor >= 0.);
        assert!(natural_factor <= 1.);
        assert_eq!(local_api_days.len(), 366);
        assert_eq!(natural_api_days.len(), 366);

        let timezone = Time::zone_from(&local_api_days[0].timezone);

        // skip averaging if possible
        if natural_factor == 0. {
            return Self::from_api_days(local_api_days, timezone);
        }

        let local_days = local_api_days.iter().map(|local_item| {
            let length = Time::from_hhmmss(&local_item.day_length);
            let center = {
                let sunrise = Time::from_military(&local_item.sunrise);
                let sunset = Time::from_military(&local_item.sunset);
                ((sunset - sunrise) / 2.0) + sunrise
            };
            LocalDay { length, center }
        }).collect::<Vec<_>>();

        let mut natural_day_lengths = natural_api_days.iter().map(|natural_item|
            Time::from_hhmmss(&natural_item.day_length)
        ).collect::<Vec<_>>();

        let local_max = local_days.iter().max_by(|a, b| a.length.cmp(&b.length)).unwrap();
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

        Self::new(timezone, day_timers.try_into().unwrap())
    }

    pub const fn timezone(&self) -> &Tz {
        &self.timezone
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
        let time = chrono_tz::Europe::Berlin.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap();
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
