use reqwest::Client;

use crate::time::Time;
use crate::timer::{day, year};

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct ResponseItem {
    /// YYYY-MM-DD
    date: String,
    /// time in military format
    sunrise: String,
    /// time in military format
    sunset: String,
    /// time in military format
    first_light: Option<String>,
    /// time in military format
    last_light: Option<String>,
    /// time in military format
    dawn: String,
    /// time in military format
    dusk: String,
    /// time in military format
    solar_noon: String,
    /// time in military format
    golden_hour: String,
    /// HH:MM:SS
    day_length: String,
    /// e.g. `"America/New_York"`, see <https://en.wikipedia.org/wiki/List_of_tz_database_time_zones>
    timezone: String,
    utc_offset: i32,
}

#[derive(Debug, serde::Deserialize)]
struct Response {
    results: Option<Vec<ResponseItem>>,
    /// e.g. `"OK"`
    status: String,
}

pub struct SunriseAPI {
    client: Client,
}

impl SunriseAPI {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn request(&self, latitude: f32, longitude: f32) -> Result<[ResponseItem; 366], ()> {
        // request leap year to get 366 response days
        let url = format!("https://api.sunrisesunset.io/json?lat={latitude}&lng={longitude}&date_start=2000-01-01&date_end=2000-12-31&time_format=military");

        let response = self.client.get(url).send().await;
        if response.is_err() {
            return Err(());
        }

        let response = response.unwrap().json::<Response>().await;
        if response.is_err() {
            return Err(());
        }

        let response = response.unwrap();
        if response.status != "OK" || response.results.is_none() || response.results.as_ref().unwrap().len() != 366 {
            return Err(());
        }

        Ok(response.results.unwrap().try_into().unwrap())
    }

    #[allow(clippy::large_stack_frames)]
    pub async fn request_year_timer(&self, latitude: f32, longitude: f32) -> Result<year::Timer, ()> {
        let response_items = self.request(latitude, longitude).await?;
        let day_timers = response_items.iter().map(|day| {
            day::Timer::new(
                Time::from_military(&day.sunrise),
                Time::from_military(&day.sunset)
            )
        }).collect::<Vec<_>>().try_into().unwrap();
        Ok(year::Timer::new(day_timers))
    }
}
