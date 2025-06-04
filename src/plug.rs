//! for shelly smart plugs compatible with the following API <https://shelly-api-docs.shelly.cloud/gen1/#shelly-plug-plugs-relay-0>

use reqwest::StatusCode;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// e.g. when url can't be reached
    SendingRequest,
    UnexpectedStatusCode(StatusCode),
}

#[derive(Debug, Clone)]
pub struct Plug {
    base_url: String,
}

impl Plug {
    /// tests given url by attempting to get power status
    pub async fn new(base_url: String) -> Result<Self, Error> {
        let plug = Self { base_url };
        plug.get_power().await?;
        Ok(plug)
    }

    pub async fn set_power(&self, power: bool) -> Result<(), Error> {
        let turn = if power { "on" } else { "off" };
        if cfg!(feature = "mock_plug") {
            log::debug!("mocking plug response: turning plug {turn}");
            return Ok(())
        }

        let base_url = &self.base_url;
        let url = format!("{base_url}/relay/0?turn={turn}");
        log::debug!("requesting url: {url}");

        let response = reqwest::get(url).await;
        if response.is_err() {
            return Err(Error::SendingRequest);
        }

        match response.unwrap().status() {
            StatusCode::OK => Ok(()),
            code => Err(Error::UnexpectedStatusCode(code))
        }
    }

    pub async fn get_power(&self) -> Result<bool, Error> {
        if cfg!(feature = "mock_plug") {
            log::debug!("mocking plug response: plug is off");
            return Ok(false)
        }

        let base_url = &self.base_url;
        let url = format!("{base_url}/relay/0");
        log::debug!("requesting url: {url}");

        let response = reqwest::get(url).await;
        if response.is_err() {
            return Err(Error::SendingRequest);
        }

        let response = response.unwrap();
        match response.status() {
            StatusCode::OK => {
                let json = response.json::<serde_json::Value>().await.unwrap();
                Ok(json["ison"].as_bool().unwrap())
            },
            code => Err(Error::UnexpectedStatusCode(code))
        }
    }

    pub fn get_url(&self) -> &str {
        self.base_url.as_str()
    }
}
