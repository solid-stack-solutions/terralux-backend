#![allow(clippy::module_name_repetitions)]

pub mod configuration;
pub mod plug;

use utoipa::OpenApi;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use utoipa_swagger_ui::SwaggerUi;
use std::{sync::Arc, net::{SocketAddr, IpAddr, Ipv4Addr}};
use axum::{response::Redirect, routing::{get, put}, http::{header, StatusCode, Method}};

use crate::constants::PORT;
use crate::state::StateWrapper;

pub type Response<T> = Result<T, (StatusCode, String)>;

fn bad_request_if(condition: bool, message: &'static str) -> Response<()> {
    if condition {
        Err((StatusCode::BAD_REQUEST, String::from(message)))
    } else {
        Ok(())
    }
}

/// start webserver. never terminates.
pub async fn start_server(state: StateWrapper) {
    // set up utoipa swagger ui
    #[derive(OpenApi)]
    #[openapi(paths(
        // functions with #[utoipa::path(...)]
        configuration::get::get_configuration,
        configuration::put::put_configuration,
        configuration::today::get::get_configuration_today,
        plug::power::get::get_plug_power,
        plug::power::put::put_plug_power,
    ))]
    struct ApiDoc;

    // configure routes
    let app = axum::Router::new()
        // api routes
        .route("/configuration", get(configuration::get::get_configuration))
        .route("/configuration", put(configuration::put::put_configuration))
        .route("/configuration/today", get(configuration::today::get::get_configuration_today))
        .route("/plug/power", put(plug::power::put::put_plug_power))
        .route("/plug/power", get(plug::power::get::get_plug_power))

        .with_state(Arc::clone(&state))

        // allow CORS from frontend
        .layer(CorsLayer::new()
            .allow_origin([
                "http://localhost:4173".parse().unwrap(), // vite dev default
                "http://localhost:5173".parse().unwrap(), // vite preview default
            ])
            .allow_methods([Method::GET, Method::PUT])
            .allow_headers([header::CONTENT_TYPE]))

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
