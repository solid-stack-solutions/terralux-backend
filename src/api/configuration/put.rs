use axum::extract;
use std::sync::Arc;

use crate::plug::Plug;
use crate::sunrise_api;
use crate::timer::year;
use crate::state::{State, StateWrapper};
use crate::api::{Response, bad_request_if};

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

    /// Latitude of geographic coordinates of terrarium, from -90° (south) to 90° (north)
    #[param(minimum = -90.0, maximum = 90.0)]
    local_latitude: f32,

    /// Longitude of geographic coordinates of terrarium, from -180° (west) to 180° (east)
    #[param(minimum = -180.0, maximum = 180.0)]
    local_longitude: f32,

    /// Latitude of geographic coordinates of the animals natural habitat, from -90° (south) to 90° (north)
    #[param(minimum = -90.0, maximum = 90.0)]
    natural_latitude: f32,

    /// Longitude of geographic coordinates of the animals natural habitat, from -180° (west) to 180° (east)
    #[param(minimum = -180.0, maximum = 180.0)]
    natural_longitude: f32,
}

#[utoipa::path(
    put, path = "/configuration",
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
) -> Response<&'static str> {
    let natural_factor = query.natural_factor;
    let local_latitude = query.local_latitude;
    let local_longitude = query.local_longitude;
    let natural_latitude = query.natural_latitude;
    let natural_longitude = query.natural_longitude;

    bad_request_if(!(   0. ..=   1.).contains(&natural_factor), "natural_factor must be between 0.0 and 1.0")?;
    bad_request_if(!(- 90. ..=  90.).contains(&local_latitude), "local_latitude must be between -90.0 and 90.0")?;
    bad_request_if(!(-180. ..= 180.).contains(&local_longitude), "local_longitude must be between -180.0 and 180.0")?;
    bad_request_if(!(- 90. ..=  90.).contains(&natural_latitude), "natural_latitude must be between -90.0 and 90.0")?;
    bad_request_if(!(-180. ..= 180.).contains(&natural_longitude), "natural_longitude must be between -180.0 and 180.0")?;

    let plug = Plug::new(query.plug_url.clone()).await;
    bad_request_if(plug.is_err(), "Could not get power state from plug using plug_url, make sure a compatible device is reachable")?;

    let local_api_days = sunrise_api::request(local_latitude, local_longitude).await?;

    let local_is_natural =
        (local_latitude  - natural_latitude ).abs() < f32::EPSILON &&
        (local_longitude - natural_longitude).abs() < f32::EPSILON;

    let natural_api_days = if local_is_natural {
        log::debug!("using API response for local location as response for natural location");
        local_api_days.clone()
    } else {
        tokio::time::sleep(sunrise_api::MIN_REQUEST_INTERVAL).await; // avoid API rate limiting
        sunrise_api::request(natural_latitude, natural_longitude).await?
    };

    let plug = plug.unwrap();
    log::info!("configured plug url: {}", plug.get_url());

    let (timezone, year_timer, local_year_timer, natural_year_timer) =
        year::Timer::from_api_days_average(natural_factor, &local_api_days, &natural_api_days);
    log::info!("configured timers");

    *state.lock().await = Some(State { natural_factor, local_latitude, local_longitude, natural_latitude, natural_longitude, plug, timezone, year_timer, local_year_timer, natural_year_timer });
    State::write_to_file(Arc::clone(&state));

    Ok("Successfully configured timers")
}
