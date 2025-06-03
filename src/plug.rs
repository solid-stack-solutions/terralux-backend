use reqwest::StatusCode;

#[derive(Debug)]
pub enum Error {
    /// e.g. when url can't be reached
    SendingRequest,
    UnexpectedStatusCode,
}

pub async fn set_power(power: bool) -> Result<(), Error> {
    // see https://shelly-api-docs.shelly.cloud/gen1/#shelly-plug-plugs-relay-0

    let base_url = "http://192.168.178.250";
    let turn = if power { "on" } else { "off" };
    let url = format!("{base_url}/relay/0?turn={turn}");
    log::debug!("requesting url: {url}");

    let response = reqwest::get(url).await;
    if response.is_err() {
        return Err(Error::SendingRequest);
    }

    match response.unwrap().status() {
        StatusCode::OK => Ok(()),
        _ => Err(Error::UnexpectedStatusCode)
    }
}
