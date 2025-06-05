use std::time::Duration;

cfg_if::cfg_if! {
    if #[cfg(feature = "demo_mode")] {
        /// to accelerate flow of time. should be > 0
        /// weird behavior when < 1?
        pub const SECONDS_PER_MINUTE: f64 = 1.;

        /// interval for checking if the current minute matches a timer
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        pub const CHECK_INTERVAL: Duration = 
            // half of SECONDS_PER_MINUTE
            Duration::from_millis((SECONDS_PER_MINUTE * 1000. / 2.) as u64);
    } else {
        /// interval for checking if the current minute matches a timer
        pub const CHECK_INTERVAL: Duration = Duration::from_secs(15);
    }
}

/// timezone to use for timers
pub const TIMEZONE: chrono_tz::Tz = chrono_tz::Europe::Berlin;

/// in [`dirs_next::data_dir()`]
pub const STATE_FILE_NAME: &str = "terralux-backend-state.json";

pub mod net {
    use std::net::{IpAddr, Ipv4Addr};
    pub const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    pub const PORT: u16 = 5000;
}
