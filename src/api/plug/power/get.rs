use axum::{extract, Json, http::StatusCode};

use crate::api::WebResponse;
use crate::state::StateWrapper;

// as json response
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct GetPlugPowerResponse {
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
pub async fn get_plug_power(
    extract::State(state): extract::State<StateWrapper>
) -> WebResponse<Json<GetPlugPowerResponse>> {
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
