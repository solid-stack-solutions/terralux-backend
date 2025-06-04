use std::time::Duration;
use axum::http::StatusCode;

use crate::web;

/// minimum interval between API requests to avoid rate limiting
pub const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
pub struct APIResponseDay {
    /// YYYY-MM-DD
    pub date: String,
    /// time in military format
    pub sunrise: String,
    /// time in military format
    pub sunset: String,
    /// time in military format
    pub first_light: Option<String>,
    /// time in military format
    pub last_light: Option<String>,
    /// time in military format
    pub dawn: String,
    /// time in military format
    pub dusk: String,
    /// time in military format
    pub solar_noon: String,
    /// time in military format
    pub golden_hour: String,
    /// HH:MM:SS
    pub day_length: String,
    /// e.g. `"America/New_York"`, see <https://en.wikipedia.org/wiki/List_of_tz_database_time_zones>
    pub timezone: String,
    pub utc_offset: i32,
}

#[derive(Debug, serde::Deserialize)]
struct APIResponse {
    #[serde(rename = "results")]
    days: Option<Vec<APIResponseDay>>,
    /// e.g. `"OK"`
    status: String,
}

/// result has exactly 366 elements
pub async fn request(latitude: f32, longitude: f32) -> web::Response<Vec<APIResponseDay>> {
    // request leap year to get 366 response days
    let url = format!("https://api.sunrisesunset.io/json?lat={latitude}&lng={longitude}&date_start=2000-01-01&date_end=2000-12-31&time_format=military");
    log::debug!("requesting latitude {latitude} and longitude {longitude}: {url}");

    // avoid reusing a reqwest::Client, as it leads to hitting the API's rate limit a lot faster
    let response = reqwest::get(url).await;
    if response.is_err() {
        return Err((StatusCode::BAD_GATEWAY, String::from("Error while sending sunrise API HTTP request")));
    }

    let response = response.unwrap();
    match response.status() {
        StatusCode::OK => (),
        StatusCode::SERVICE_UNAVAILABLE => {
            log::warn!("sunrise API rate limit reached");
            return Err((StatusCode::TOO_MANY_REQUESTS, String::from("Reached sunrise API request rate limit")));
        },
        code =>
            return Err((StatusCode::BAD_GATEWAY, format!("Sunrise API unexpectedly responded with HTTP status code {code}"))),
    }

    let response_text = response.text().await.unwrap();
    let response = serde_json::from_str::<APIResponse>(&response_text);
    if response.is_err() {
        log::warn!("failed to deserialize the following response: {}", response_text);
        return Err((StatusCode::BAD_GATEWAY, String::from("Error while parsing sunrise API response")));
    }

    let response = response.unwrap();
    if response.status != "OK" {
        return Err((StatusCode::BAD_GATEWAY, format!("Sunrise API responded with \"{}\" instead of \"OK\"", response.status)));
    }
    if response.days.is_none() {
        return Err((StatusCode::BAD_GATEWAY, String::from("Sunrise API responded \"OK\" without any data")));
    }

    let days = response.days.unwrap();
    if days.len() != 366 {
        return Err((StatusCode::BAD_GATEWAY, format!("Sunrise API response had data for {} instead of 366 days", days.len())));
    }

    Ok(days)
}
