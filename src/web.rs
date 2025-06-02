use std::sync::Arc;
use tokio::sync::Mutex;
use axum::{
    extract::{self, State},
    http::StatusCode
};

use crate::timer::year;

type Response<T> = Result<T, (StatusCode, &'static str)>;

#[derive(
    Debug,
    serde::Deserialize, // from axum::Json
    utoipa::ToSchema,  // to display in swagger-ui
    utoipa::IntoParams, // ???
)]
struct PutState {
    /// average sunrise/sunset times between local ones (0.0) and ones from the natural habitat (1.0)
    #[schema(minimum = 0.0, maximum = 1.0)]
    natural_factor: f32
}

#[utoipa::path(
    put,
    path = "/state",
    params(PutState),
    responses(
        (status = 200, description = "Successfully put state.", body = PutState),
        (status = 400, description = "Request did not match expected structure."),
    ),
)]
async fn put_state(
    State(state): State<Arc<Mutex<Option<year::Timer>>>>,
    extract::Json(new_state): extract::Json<PutState>
) -> Response<&'static str> {
    Ok("response")
}

/// start webserver. never terminates.
pub async fn start_server(year_timer: Arc<Mutex<Option<year::Timer>>>) {
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
            put_state,
        ),
        components(schemas(
            // enums/structs with #[derive(utoipa::ToSchema)]
            PutState,
        ))
    )]
    struct ApiDoc;

    // configure routes
    let app = axum::Router::new()
        // api routes
        .route("/state", put(put_state))
            .with_state(Arc::clone(&year_timer))

        // temporarily redirect root to swagger ui
        .route("/", get(|| async { Redirect::temporary("/swagger-ui") }))
        // swagger ui
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/openapi.json", ApiDoc::openapi()));

    let address = std::net::SocketAddr::new(LOCALHOST, PORT);
    println!("starting server on http://{address} ...");
    axum::serve(TcpListener::bind(address).await.unwrap(), app).await.unwrap();
}
