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
    pub fn new(on_time: Time, off_time: Time) -> Self {
        assert_ne!(on_time, off_time);
        Self { on_time, off_time }
    }

    pub const fn on_time(&self) -> &Time {
        &self.on_time
    }

    pub const fn off_time(&self) -> &Time {
        &self.off_time
    }
}

impl std::fmt::Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ on: {}, off: {} }}", self.on_time, self.off_time)
    }
}
