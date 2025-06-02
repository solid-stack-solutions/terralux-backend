use crate::time::Time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timer {
    /// time to turn the plug on
    on_time: Time,
    /// time to turn the plug off
    off_time: Time,
}

impl Timer {
    /// with times to turn the plug on/off
    fn new(on_time: Time, off_time: Time) -> Self {
        assert_ne!(on_time, off_time);
        Self { on_time, off_time }
    }
}
