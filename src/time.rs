#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

    pub fn now() -> Self {
        use chrono::{Utc, Timelike};
        use crate::constants::TIMEZONE;

        let now = Utc::now().with_timezone(&TIMEZONE);
        Self::new(
            now.hour().try_into().unwrap(),
            now.minute().try_into().unwrap(),
        )
    }
}

impl std::ops::Sub for Time {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let negative_rhs = Self { hour: -rhs.hour, minute: -rhs.minute };
        self + negative_rhs
    }
}

impl std::ops::Add for Time {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let total_minutes_self = i16::from(self.minute) + (i16::from(self.hour) * 60);
        let total_minutes_rhs  = i16::from( rhs.minute) + (i16::from( rhs.hour) * 60);
        let total_minutes = total_minutes_self + total_minutes_rhs;

        // still in range of same day
        assert!(total_minutes >= 0);
        assert!(total_minutes < 24 * 60);

        let hour = total_minutes / 60;
        let minute = total_minutes - (hour * 60);

        Self::new(
            hour.try_into().unwrap(),
            minute.try_into().unwrap()
        )
    }
}
