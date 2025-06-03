#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time {
    /// between 0 and 23 
    hour: i8,
    /// between 0 and 59 
    minute: i8,
}

impl Time {
    pub const fn new(hour: i8, minute: i8) -> Self {
        assert!(hour >=  0);
        assert!(hour <  24);
        assert!(minute >=  0);
        assert!(minute <  60);
        Self { hour, minute }
    }

    /// from military time string like "1800"
    pub fn from_military(military_time: &str) -> Self {
        assert_eq!(military_time.len(), 4);
        let (hour, minute) = military_time.split_at(2);
        Self::new(hour.parse().unwrap(), minute.parse().unwrap())
    }

    /// from time string with format "HH:MM:SS"
    pub fn from_hhmmss(hhmmss_time: &str) -> Self {
        let parts = hhmmss_time.split(':').collect::<Vec<_>>();
        Self::new(parts[0].parse().unwrap(), parts[1].parse().unwrap())
    }

    pub fn now() -> Self {
        use chrono::{Utc, Timelike};
        use crate::constants::TIMEZONE;

        let now = Utc::now().with_timezone(&TIMEZONE);
        Self::new(
            now.hour().try_into().unwrap(),
            now.minute().try_into().unwrap(),
        )
    }

    fn minutes(self) -> i16 {
        i16::from(self.minute) + (i16::from(self.hour) * 60)
    }

    fn from_minutes(minutes: i16) -> Self {
        // still in range of same day
        assert!(minutes >= 0);
        assert!(minutes < 24 * 60);

        let hour = minutes / 60;
        let minute = minutes - (hour * 60);

        Self::new(
            hour.try_into().unwrap(),
            minute.try_into().unwrap()
        )
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}

// order by total minutes
impl Ord for Time {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.minutes().cmp(&other.minutes())
    }
}
impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::ops::Sub for Time {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let minutes_self = self.minutes();
        let minutes_rhs  = rhs.minutes();
        Self::from_minutes(minutes_self - minutes_rhs)
    }
}

impl std::ops::Add for Time {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let minutes_self = self.minutes();
        let minutes_rhs  = rhs.minutes();
        Self::from_minutes(minutes_self + minutes_rhs)
    }
}

impl std::ops::Div<f32> for Time {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        let minutes_self = f32::from(self.minutes());
        #[allow(clippy::cast_possible_truncation)]
        let minutes = (minutes_self / rhs) as i16;
        Self::from_minutes(minutes)
    }
}

impl std::ops::Mul<f32> for Time {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        let minutes_self = f32::from(self.minutes());
        #[allow(clippy::cast_possible_truncation)]
        let minutes = (minutes_self * rhs) as i16;
        Self::from_minutes(minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::Time;
    const ZERO: Time = Time::new(0, 0);

    #[test]
    fn add_zero_1() {
        assert_eq!(ZERO + ZERO, ZERO);
    }

    #[test]
    fn add_zero_2() {
        let time = Time::new(10, 12);
        assert_eq!(time + ZERO, time);
    }

    #[test]
    fn add() {
        assert_eq!(Time::new(8, 12) + Time::new(3, 59), Time::new(12, 11));
    }

    #[test]
    fn sub_zero_1() {
        assert_eq!(ZERO - ZERO, ZERO);
    }

    #[test]
    fn sub_zero_2() {
        let time = Time::new(10, 12);
        assert_eq!(time - ZERO, time);
    }

    #[test]
    fn sub() {
        assert_eq!(Time::new(8, 15) - Time::new(1, 20), Time::new(6, 55));
    }

    #[test]
    fn from_military() {
        assert_eq!(Time::from_military("1612"), Time::new(16, 12));
    }

    #[test]
    fn div() {
        assert_eq!(Time::new(3, 20) / 2., Time::new(1, 40));
    }

    #[test]
    fn mul() {
        assert_eq!(Time::new(3, 20) * 1.2, Time::new(4, 0));
    }

    #[test]
    fn from_hhmmss() {
        assert_eq!(Time::from_hhmmss("18:42:02"), Time::new(18, 42));
        assert_eq!(Time::from_hhmmss("18:42:59"), Time::new(18, 42));
    }
}
