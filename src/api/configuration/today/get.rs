use axum::{extract, Json, http::StatusCode};

use crate::timer::day;
use crate::api::WebResponse;
use crate::state::StateWrapper;

#[utoipa::path(
    get, path = "/configuration/today",
    responses(
        (status = 200, description = "Got todays configuration", body = day::Timer),
        (status = 409, description = "Not yet configured"),
    ),
)]
pub async fn endpoint(
    extract::State(state): extract::State<StateWrapper>
) -> WebResponse<Json<day::Timer>> {
    state.lock().await.as_ref().map_or_else(
        || Err((StatusCode::CONFLICT, String::from("Not yet configured, consider calling /configuration first"))),
        |state| Ok(Json(*state.year_timer.for_today(state.timezone)))
    )
}
