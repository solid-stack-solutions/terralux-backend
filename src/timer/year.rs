use super::day;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timer {
    /// does not include leap day
    day_timers: [day::Timer; 365]
}
