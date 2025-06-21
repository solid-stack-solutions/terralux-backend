use std::sync::Arc;
use axum::{extract, Json, http::StatusCode};

use crate::plug::Plug;
use crate::sunrise_api;
use crate::timer::{day, year};
use crate::state::{State, StateWrapper};

pub type Response<T> = Result<T, (StatusCode, String)>;

fn bad_request_if(condition: bool, message: &'static str) -> Response<()> {
    if condition {
        Err((StatusCode::BAD_REQUEST, String::from(message)))
    } else {
        Ok(())
    }
}

// from query parameters
#[derive(utoipa::IntoParams, serde::Deserialize)]
struct PutConfigurationQuery {
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
async fn put_configuration(
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

// from query parameters
#[derive(utoipa::ToSchema, serde::Serialize)]
struct GetConfigurationResponse {
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
        (status = 200, description = "Got configuration", body = GetConfigurationResponse),
        (status = 409, description = "Not yet configured"),
    ),
)]
#[allow(clippy::significant_drop_tightening)]
async fn get_configuration(
    extract::State(state): extract::State<StateWrapper>
) -> Response<Json<GetConfigurationResponse>> {
    let state = state.lock().await;
    if state.is_none() {
        return Err((StatusCode::CONFLICT, String::from("Not yet configured, consider calling /configuration first")));
    }
    let state = state.as_ref().unwrap();

    Ok(Json(GetConfigurationResponse {
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

#[utoipa::path(
    get, path = "/configuration/today",
    responses(
        (status = 200, description = "Got todays configuration", body = day::Timer),
        (status = 409, description = "Not yet configured"),
    ),
)]
async fn get_configuration_today(
    extract::State(state): extract::State<StateWrapper>
) -> Response<Json<day::Timer>> {
    state.lock().await.as_ref().map_or_else(
        || Err((StatusCode::CONFLICT, String::from("Not yet configured, consider calling /configuration first"))),
        |state| Ok(Json(*state.year_timer.for_today(state.timezone)))
    )
}

// from query parameters
#[derive(utoipa::IntoParams, serde::Deserialize)]
struct PutPlugPowerQuery {
    /// Whether to turn the plug on (`true`) or off (`false`)
    power: bool,
}

#[utoipa::path(
    put, path = "/plug/power",
    params(PutPlugPowerQuery),
    responses(
        (status = 200, description = "Successfully set plugs power state"),
        (status = 400, description = "Query parameters did not match expected structure"),
        (status = 409, description = "Plug not yet configured"),
        (status = 502, description = "Unexpected response from plug"),
    ),
)]
#[allow(clippy::significant_drop_tightening)]
async fn put_plug_power(
    extract::State(state): extract::State<StateWrapper>,
    extract::Query(query): extract::Query<PutPlugPowerQuery>
) -> Response<String> {
    use crate::plug::Error;

    let state = state.lock().await;
    if state.is_none() {
        return Err((StatusCode::CONFLICT, String::from("Plug not yet configured, consider calling /configuration first")));
    }

    let plug = &state.as_ref().unwrap().plug;
    match plug.set_power(query.power).await {
        Ok(()) => Ok(format!("Successfully turned plug {}", if query.power { "on" } else { "off" })),
        Err(error) => {
            let message = match error {
                Error::SendingRequest => format!("Error while sending HTTP request to plug, make sure it's available on {}", plug.get_url()),
                Error::UnexpectedStatusCode(code) => format!("Plug unexpectedly responded with HTTP status code {code}"),
            };
            Err((StatusCode::BAD_GATEWAY, message))
        }
    }
}

// as json response
#[derive(utoipa::ToSchema, serde::Serialize)]
struct GetPlugPowerResponse {
    /// Whether the plug is on (`true`) or off (`false`)
    power: bool,
}

#[utoipa::path(
    get, path = "/plug/power",
    responses(
        (status = 200, description = "Got plugs power state (`true` meaning \"on\" and `false` meaning \"off\")", body = GetPlugPowerResponse),
        (status = 409, description = "Plug not yet configured"),
        (status = 502, description = "Unexpected response from plug"),
    ),
)]
#[allow(clippy::significant_drop_tightening)]
async fn get_plug_power(
    extract::State(state): extract::State<StateWrapper>
) -> Response<Json<GetPlugPowerResponse>> {
    use crate::plug::Error;

    let state = state.lock().await;
    if state.is_none() {
        return Err((StatusCode::CONFLICT, String::from("Plug not yet configured, consider calling /configuration first")));
    }

    let plug = &state.as_ref().unwrap().plug;
    match plug.get_power().await {
        Ok(power) => Ok(Json(GetPlugPowerResponse { power })),
        Err(error) => {
            let message = match error {
                Error::SendingRequest => format!("Error while sending HTTP request to plug, make sure it's available on {}", plug.get_url()),
                Error::UnexpectedStatusCode(code) => format!("Plug unexpectedly responded with HTTP status code {code}"),
            };
            Err((StatusCode::BAD_GATEWAY, message))
        }
    }
}

/// start webserver. never terminates.
pub async fn start_server(state: StateWrapper) {
    use utoipa::OpenApi;
    use tokio::net::TcpListener;
    use utoipa_swagger_ui::SwaggerUi;
    use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    use axum::{response::Redirect, routing::{get, put}};

    use crate::constants::PORT;

    // set up utoipa swagger ui
    #[derive(OpenApi)]
    #[openapi(paths(
        // functions with #[utoipa::path(...)]
        put_configuration,
        get_configuration,
        get_configuration_today,
        put_plug_power,
        get_plug_power,
    ))]
    struct ApiDoc;

    // configure routes
    let app = axum::Router::new()
        // api routes
        .route("/configuration", put(put_configuration))
        .route("/configuration", get(get_configuration))
        .route("/configuration/today", get(get_configuration_today))
        .route("/plug/power", put(put_plug_power))
        .route("/plug/power", get(get_plug_power))

        .with_state(Arc::clone(&state))

        // temporarily redirect root to swagger ui
        .route("/", get(|| async { Redirect::temporary("/swagger-ui") }))
        // swagger ui
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/openapi.json", ApiDoc::openapi()));

    // visible on localhost and from outside
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), PORT);
    let tcp_listener = TcpListener::bind(address).await;
    if let Err(ref error) = tcp_listener {
        if error.kind() == std::io::ErrorKind::AddrInUse {
            log::error!("port {PORT} is already in use, is this server already running?");
            std::process::exit(1);
        } else {
            panic!("{error:?}");
        }
    }

    log::info!("listening on port {PORT} on all interfaces (local access: http://localhost:{PORT})");
    axum::serve(tcp_listener.unwrap(), app).await.unwrap();
}
