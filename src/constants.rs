use std::time::Duration;

/// interval for checking if the current minute matches a timer
pub const CHECK_INTERVAL: Duration = Duration::from_secs(15);

/// timezone to use for timers
pub const TIMEZONE: chrono_tz::Tz = chrono_tz::Europe::Berlin;

/// in [`dirs_next::data_dir()`]
pub const STATE_FILE_NAME: &str = "terralux-backend-state.json";

pub mod net {
    use std::net::{IpAddr, Ipv4Addr};
    pub const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    pub const PORT: u16 = 5000;
}
