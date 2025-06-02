use std::time::Duration;

/// interval for checking if the current minute matches a timer
pub const CHECK_INTERVAL: Duration = Duration::from_secs(5);

/// timezone to use for timers
pub const TIMEZONE: chrono_tz::Tz = chrono_tz::Europe::Berlin;

pub mod net {
    use std::net::{IpAddr, Ipv4Addr};
    pub const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    pub const PORT: u16 = 5000;
}
