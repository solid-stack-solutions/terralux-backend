use axum::{extract, Json, http::StatusCode};

use crate::timer::day;
use crate::api::WebResponse;
use crate::state::StateWrapper;

// as json response
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct Response {
    /// Average sunrise/sunset times between local ones (`0.0`) and ones from the natural habitat (`1.0`)
    #[schema(minimum = 0.0, maximum = 1.0, example = 0.5)]
    natural_factor: f32,

    /// Latitude of geographic coordinates of terrarium, from -90° (south) to 90° (north)
    #[schema(minimum = -90.0, maximum = 90.0)]
    local_latitude: f32,

    /// Longitude of geographic coordinates of terrarium, from -180° (west) to 180° (east)
    #[schema(minimum = -180.0, maximum = 180.0)]
    local_longitude: f32,

    /// Latitude of geographic coordinates of the animals natural habitat, from -90° (south) to 90° (north)
    #[schema(minimum = -90.0, maximum = 90.0)]
    natural_latitude: f32,

    /// Longitude of geographic coordinates of the animals natural habitat, from -180° (west) to 180° (east)
    #[schema(minimum = -180.0, maximum = 180.0)]
    natural_longitude: f32,

    /// URL to Shelly smart plug to control
    #[schema(example = "http://192.168.178.123")]
    plug_url: String,

    /// IANA timezone to use for timer activations
    #[schema(example = "Europe/Berlin")]
    timezone: String,

    /// Timers to turn plug on/off every day, computed with given `natural_factor`, including possible leap day
    #[serde(with = "serde_big_array::BigArray")]
    #[schema(min_items = 366, max_items = 366)]
    computed_timers: [day::Timer; 366],

    /// Theoretical timers to turn plug on/off every day if `natural_factor` was `0.0`, including possible leap day
    #[serde(with = "serde_big_array::BigArray")]
    #[schema(min_items = 366, max_items = 366)]
    local_timers: [day::Timer; 366],

    /// Theoretical timers to turn plug on/off every day if `natural_factor` was `1.0`, including possible leap day
    #[serde(with = "serde_big_array::BigArray")]
    #[schema(min_items = 366, max_items = 366)]
    natural_timers: [day::Timer; 366],
}

#[utoipa::path(
    get, path = "/configuration",
    responses(
        (status = 200, description = "Got configuration", body = Response),
        (status = 409, description = "Not yet configured"),
    ),
)]
#[allow(clippy::significant_drop_tightening)]
pub async fn endpoint(
    extract::State(state): extract::State<StateWrapper>
) -> WebResponse<Json<Response>> {
    let state = state.lock().await;
    if state.is_none() {
        return Err((StatusCode::CONFLICT, String::from("Not yet configured, consider calling /configuration first")));
    }
    let state = state.as_ref().unwrap();

    Ok(Json(Response {
        natural_factor: state.natural_factor,
        local_latitude: state.local_latitude,
        local_longitude: state.local_longitude,
        natural_latitude: state.natural_latitude,
        natural_longitude: state.natural_longitude,
        plug_url: state.plug.get_url().to_string(),
        timezone: state.timezone.to_string(),
        computed_timers: *state.year_timer.day_timers(),
        local_timers: *state.local_year_timer.day_timers(),
        natural_timers: *state.natural_year_timer.day_timers(),
    }))
}
