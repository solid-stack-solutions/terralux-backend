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

    pub const fn for_today(&self) -> &day::Timer {
        // TODO implement
        &self.day_timers[0]
    }
}
