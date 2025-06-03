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

    /// result has exactly 366 elements
    async fn request(&self, latitude: f32, longitude: f32) -> Result<Vec<ResponseItem>, ()> {
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

        Ok(response.results.unwrap())
    }

    #[allow(clippy::large_stack_frames)]
    pub async fn request_year_timer(&self, natural_factor: f32, local_latitude: f32, local_longitude: f32, natural_latitude: f32, natural_longitude: f32) -> Result<year::Timer, ()> {
        struct LocalDay {
            length: Time,
            /// exactly in between sunrise and sunset
            center: Time,
        }

        let local_items = self.request(local_latitude, local_longitude).await?;
        let local_days = local_items.iter().map(|local_item| {
            let length = Time::from_hhmmss(&local_item.day_length);
            let center = {
                let sunrise = Time::from_military(&local_item.sunrise);
                let sunset = Time::from_military(&local_item.sunset);
                ((sunset - sunrise) / 2.0) + sunrise
            };
            LocalDay { length, center }
        }).collect::<Vec<_>>();

        let natural_items = self.request(natural_latitude, natural_longitude).await?;
        let natural_day_lengths = natural_items.iter().map(|natural_item|
            Time::from_hhmmss(&natural_item.day_length)
        ).collect::<Vec<_>>();

        // TODO shift natural_day_lengths based on longest day

        let day_timers = local_days.iter()
            .zip(natural_day_lengths.iter())
            .map(|(local_day, natural_day_length)| {
                let averaged_day_length = (*natural_day_length * natural_factor)
                    + (local_day.length * (1. - natural_factor));
                let on  = local_day.center - (averaged_day_length / 2.);
                let off = local_day.center + (averaged_day_length / 2.);
                log::trace!("center: {}, local length: {}, natural length: {} => on: {}, off {}", local_day.center, local_day.length, natural_day_length, on, off);
                day::Timer::new(on, off)
            })
            .collect::<Vec<_>>();

        Ok(year::Timer::new(day_timers.try_into().unwrap()))
    }
}
