use axum::{extract, http::StatusCode};

use crate::api::WebResponse;
use crate::state::StateWrapper;

// from query parameters
#[derive(utoipa::IntoParams, serde::Deserialize)]
pub struct PutPlugPowerQuery {
    /// Whether to turn the plug on (`true`) or off (`false`)
    power: bool,
}

#[utoipa::path(
    put, path = "/plug/power",
    tag = "Plug",
    params(PutPlugPowerQuery),
    responses(
        (status = 200, description = "Successfully set plugs power state"),
        (status = 400, description = "Query parameters did not match expected structure"),
        (status = 409, description = "Plug not yet configured"),
        (status = 502, description = "Unexpected response from plug"),
    ),
)]
#[allow(clippy::significant_drop_tightening)]
pub async fn put_plug_power(
    extract::State(state): extract::State<StateWrapper>,
    extract::Query(query): extract::Query<PutPlugPowerQuery>
) -> WebResponse<String> {
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
