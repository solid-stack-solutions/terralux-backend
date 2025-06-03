use chrono::{DateTime, NaiveDate, Datelike, Utc};
use chrono_tz::Tz;

use super::day;
use crate::time::Time;
use crate::constants::TIMEZONE;
use crate::sunrise_api::APIResponseDay;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timer {
    /// includes leap day
    day_timers: [day::Timer; 366]
}

impl Timer {
    /// day timers include leap day
    #[allow(clippy::large_types_passed_by_value)]
    pub const fn new(day_timers: [day::Timer; 366]) -> Self {
        Self { day_timers }
    }

    pub fn for_today(&self) -> &day::Timer {
        let now = Utc::now().with_timezone(&TIMEZONE);

        &self.day_timers[Self::index(now)]
    }

    pub fn from_api_days(api_days: &[APIResponseDay]) -> Self {
        todo!()
    }

    pub fn from_api_days_average(natural_factor: f32, local_api_days: &[APIResponseDay], natural_api_days: &[APIResponseDay]) -> Self {
        struct LocalDay {
            length: Time,
            /// exactly in between sunrise and sunset
            center: Time,
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

        let natural_day_lengths = natural_api_days.iter().map(|natural_item|
            Time::from_hhmmss(&natural_item.day_length)
        ).collect::<Vec<_>>();

        // TODO shift natural_day_lengths based on longest day

        let day_timers = local_days.iter()
            .zip(natural_day_lengths.iter())
            .map(|(local_day, natural_day_length)| {
                let averaged_day_length = (*natural_day_length * natural_factor)
                    + (local_day.length * (1. - natural_factor));
                let on  = local_day.center - (averaged_day_length / 2.);
                let off = local_day.center + (averaged_day_length / 2.);
                log::trace!("center: {}, local length: {}, natural length: {} => on: {}, off {}", local_day.center, local_day.length, natural_day_length, on, off);
                day::Timer::new(on, off)
            })
            .collect::<Vec<_>>();

        Self::new(day_timers.try_into().unwrap())
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
        let time = TIMEZONE.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap();
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
