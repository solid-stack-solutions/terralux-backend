use crate::time::Time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timer {
    /// time to turn the plug on
    on_time: Time,
    /// time to turn the plug off
    off_time: Time,
}
