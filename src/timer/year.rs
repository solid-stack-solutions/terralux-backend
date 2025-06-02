use super::day;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timer {
    /// does not include leap day
    day_timers: [day::Timer; 365]
}

impl Timer {
    /// day timers don't include leap day
    #[allow(clippy::large_types_passed_by_value)]
    pub const fn new(day_timers: [day::Timer; 365]) -> Self {
        Self { day_timers }
    }

    pub fn for_today(&self) -> &day::Timer {
        use chrono::{Utc, Datelike};
        use crate::constants::TIMEZONE;

        // may be 365 in leap years
        let day = Utc::now().with_timezone(&TIMEZONE).ordinal0();

        let index = match day {
            // treat last day of leap year like second last
            // to stay in range of day timer index (max. 364)
            365 => 364,
            _ => day,
        };

        &self.day_timers[index as usize]
    }
}
