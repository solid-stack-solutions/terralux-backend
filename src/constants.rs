use std::time::Duration;

cfg_if::cfg_if! {
    if #[cfg(feature = "demo_mode")] {
        /// to accelerate flow of time. should be > 0.
        /// in real time, this would be 1440 (24h * 60min/h).
        pub const MINUTES_PER_DAY: f32 = 1.;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        pub const MILLISECONDS_PER_MINUTE: u16 = (MINUTES_PER_DAY / 24. * 1000.) as u16;

        /// interval for checking if the current minute matches a timer
        pub const CHECK_INTERVAL: Duration = Duration::from_millis((MILLISECONDS_PER_MINUTE / 2) as u64);
    } else {
        /// interval for checking if the current minute matches a timer
        pub const CHECK_INTERVAL: Duration = Duration::from_secs(15);
    }
}

/// in [`dirs_next::data_dir()`]
pub const STATE_FILE_NAME: &str = "terralux-backend-state.json";

pub const PORT: u16 = 5000;
