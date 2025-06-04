use std::sync::Arc;
use tokio::sync::Mutex;
use axum::{
    extract::{Query, State},
    http::StatusCode
};

use crate::plug::Plug;
use crate::sunrise_api;
use crate::timer::year;

pub type Response<T> = Result<T, (StatusCode, String)>;
type StateYearTimer = Arc<Mutex<Option<year::Timer>>>;
type StatePlug = Arc<Mutex<Option<Plug>>>;

fn bad_request_if(condition: bool, message: &'static str) -> Response<()> {
    if condition {
        Err((StatusCode::BAD_REQUEST, String::from(message)))
    } else {
        Ok(())
    }
}

#[derive(
    Debug,
    // from query parameters
    utoipa::IntoParams, serde::Deserialize,
)]
struct PutConfigurationQuery {
    /// URL to Shelly smart plug compatible with [this API](https://shelly-api-docs.shelly.cloud/gen1/#shelly-plug-plugs-relay-0)
    /// without a trailing slash, e.g. `http://192.168.178.123`
    plug_url: String,

    /// Average sunrise/sunset times between local ones (`0.0`) and ones from the natural habitat (`1.0`)
    #[param(minimum = 0.0, maximum = 1.0)]
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
    State(state): State<(StateYearTimer, StatePlug)>,
    Query(query): Query<PutConfigurationQuery>
) -> Response<&'static str> {
    bad_request_if(query.natural_factor < 0. || query.natural_factor > 1., "natural_factor must be between 0.0 and 1.0")?;
    bad_request_if(query.local_latitude < -90. || query.local_latitude > 90., "local_latitude must be between -90.0 and 90.0")?;
    bad_request_if(query.local_longitude < -180. || query.local_longitude > 180., "local_longitude must be between -180.0 and 180.0")?;
    bad_request_if(query.natural_latitude < -90. || query.natural_latitude > 90., "natural_latitude must be between -90.0 and 90.0")?;
    bad_request_if(query.natural_longitude < -180. || query.natural_longitude > 180., "natural_longitude must be between -180.0 and 180.0")?;
    let plug = Plug::new(query.plug_url.clone()).await;
    bad_request_if(plug.is_err(), "Could not get power state from plug using plug_url, make sure a compatible device is reachable")?;

    let (state_year_timer, state_plug) = state;
    *state_plug.lock().await = Some(plug.unwrap());
    log::info!("configured plug url: {}", query.plug_url);

    let local_api_days = sunrise_api::request(query.local_latitude, query.local_longitude).await?;
    tokio::time::sleep(sunrise_api::MIN_REQUEST_INTERVAL).await; // avoid API rate limiting
    let natural_api_days = sunrise_api::request(query.natural_latitude, query.natural_longitude).await?;

    let year_timer = year::Timer::from_api_days_average(query.natural_factor, &local_api_days, &natural_api_days);
    *state_year_timer.lock().await = Some(year_timer);
    log::info!("configured timers");

    Ok("Successfully configured timers")
}

/// start webserver. never terminates.
pub async fn start_server(year_timer: StateYearTimer, plug: StatePlug) {
    use utoipa::OpenApi;
    use tokio::net::TcpListener;
    use utoipa_swagger_ui::SwaggerUi;
    use axum::{response::Redirect, routing::{get, put}};

    use crate::constants::net::{LOCALHOST, PORT};

    // set up utoipa swagger ui
    #[derive(OpenApi)]
    #[openapi(
        paths(
            // functions with #[utoipa::path(...)]
            put_configuration,
        ),
        // enums/structs with #[derive(utoipa::ToSchema)]
        //components(schemas( ... ))
    )]
    struct ApiDoc;

    // configure routes
    let app = axum::Router::new()
        // api routes
        .route("/configuration", put(put_configuration))
            .with_state((Arc::clone(&year_timer), Arc::clone(&plug)))

        // temporarily redirect root to swagger ui
        .route("/", get(|| async { Redirect::temporary("/swagger-ui") }))
        // swagger ui
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/openapi.json", ApiDoc::openapi()));

    let address = std::net::SocketAddr::new(LOCALHOST, PORT);
    log::info!("starting server on http://{address}");

    let tcp_listener = TcpListener::bind(address).await;
    if let Err(ref error) = tcp_listener {
        if error.kind() == std::io::ErrorKind::AddrInUse {
            log::error!("address http://{address} is already in use, is this server already running?");
            std::process::exit(1);
        } else {
            panic!("{error:?}");
        }
    }

    axum::serve(tcp_listener.unwrap(), app).await.unwrap();
}
