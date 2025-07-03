use axum::extract;
use std::sync::Arc;

use crate::plug::Plug;
use crate::timer::year;
use crate::sunrise_api::request;
use crate::state::{State, StateWrapper};
use crate::api::{WebResponse, bad_request_if};
use crate::constants::{MIN_SUNRISE_API_REQUEST_INTERVAL, ABS_POLAR_CIRCLE_LAT};

// from query parameters
#[derive(utoipa::IntoParams, serde::Deserialize)]
pub struct PutConfigurationQuery {
    /// URL to Shelly smart plug compatible with [this API](https://shelly-api-docs.shelly.cloud/gen1/#shelly-plug-plugs-relay-0)
    /// without a trailing slash
    #[param(example = "http://192.168.178.123")]
    plug_url: String,

    /// Average sunrise/sunset times between local ones (`0.0`) and ones from the natural habitat (`1.0`)
    #[param(minimum = 0.0, maximum = 1.0, example = 0.5)]
    natural_factor: f32,

    /// Latitude of geographic coordinates of terrarium, from -65° (south) to 65° (north) (limits exclusive)
    #[param(minimum = -65.0, maximum = 65.0)]
    local_latitude: f32,

    /// Longitude of geographic coordinates of terrarium, from -180° (west) to 180° (east)
    #[param(minimum = -180.0, maximum = 180.0)]
    local_longitude: f32,

    /// Latitude of geographic coordinates of the animals natural habitat, from -65° (south) to 65° (north) (limits exclusive)
    #[param(minimum = -65.0, maximum = 65.0)]
    natural_latitude: f32,

    /// Longitude of geographic coordinates of the animals natural habitat, from -180° (west) to 180° (east)
    #[param(minimum = -180.0, maximum = 180.0)]
    natural_longitude: f32,
}

#[utoipa::path(
    put, path = "/configuration",
    tag = "Configuration",
    params(PutConfigurationQuery),
    responses(
        (status = 200, description = "Successfully configured timers"),
        (status = 400, description = "Query parameters did not match expected structure"),
        (status = 429, description = "Reached sunrise API request rate limit"),
        (status = 502, description = "Unexpected response from sunrise API"),
    ),
)]
pub async fn put_configuration(
    extract::State(state): extract::State<StateWrapper>,
    extract::Query(query): extract::Query<PutConfigurationQuery>
) -> WebResponse<&'static str> {
    let natural_factor = query.natural_factor;
    let local_latitude = query.local_latitude;
    let local_longitude = query.local_longitude;
    let natural_latitude = query.natural_latitude;
    let natural_longitude = query.natural_longitude;

    bad_request_if(!(   0. ..=   1.).contains(&natural_factor), "natural_factor must be between 0.0 and 1.0".to_string())?;
    bad_request_if(!(-180. ..= 180.).contains(&local_longitude), "local_longitude must be between -180.0 and 180.0".to_string())?;
    bad_request_if(!(-180. ..= 180.).contains(&natural_longitude), "natural_longitude must be between -180.0 and 180.0".to_string())?;
    bad_request_if(  local_latitude <= -ABS_POLAR_CIRCLE_LAT ||   local_latitude >= ABS_POLAR_CIRCLE_LAT,
        format!("local_latitude must be between -{ABS_POLAR_CIRCLE_LAT:.1} and {ABS_POLAR_CIRCLE_LAT:.1} (limits exclusive)"))?;
    bad_request_if(natural_latitude <= -ABS_POLAR_CIRCLE_LAT || natural_latitude >= ABS_POLAR_CIRCLE_LAT,
        format!("natural_latitude must be between -{ABS_POLAR_CIRCLE_LAT:.1} and {ABS_POLAR_CIRCLE_LAT:.1} (limits exclusive)"))?;

    let plug = Plug::new(query.plug_url.clone()).await;
    bad_request_if(plug.is_err(), "Could not get power state from plug using plug_url, make sure a compatible device is reachable".to_string())?;

    let local_api_days = request(local_latitude, local_longitude).await?;

    let local_is_natural =
        (local_latitude  - natural_latitude ).abs() < f32::EPSILON &&
        (local_longitude - natural_longitude).abs() < f32::EPSILON;

    let natural_api_days = if local_is_natural {
        log::debug!("using API response for local location as response for natural location");
        local_api_days.clone()
    } else {
        tokio::time::sleep(MIN_SUNRISE_API_REQUEST_INTERVAL).await; // avoid API rate limiting
        request(natural_latitude, natural_longitude).await?
    };

    let plug = plug.unwrap();
    log::info!("configured plug url: {}", plug.get_url());

    let (timezone, year_timer, local_year_timer, natural_year_timer) =
        year::Timer::from_api_days_average(natural_factor, &local_api_days, &natural_api_days)?;
    log::info!("configured timers");

    *state.lock().await = Some(State { natural_factor, local_latitude, local_longitude, natural_latitude, natural_longitude, plug, timezone, year_timer, local_year_timer, natural_year_timer });
    State::write_to_file(Arc::clone(&state));

    Ok("Successfully configured timers")
}
