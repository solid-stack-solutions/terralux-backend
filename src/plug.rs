//! for shelly smart plugs compatible with the following API <https://shelly-api-docs.shelly.cloud/gen1/#shelly-plug-plugs-relay-0>

use reqwest::StatusCode;
use std::time::Duration;

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

    /// retry on error for about 30min, with increasing interval between requests (up to 1min)
    pub async fn set_power_with_retry(&self, power: bool) {
        if self.set_power(power).await.is_ok() {
            return;
        }

        // about  5min for 10 linear increase interval retries +
        // about 25min for 25 1min interval retries = 35 retries
        for retry in 1 ..= 5 {
            // linear increase until maxing out at 60
            let seconds = (6 * retry).min(60);
            log::warn!("failed to set plugs power state, attempting retry {retry} in {seconds} seconds");
            tokio::time::sleep(Duration::from_secs(seconds)).await;

            if self.set_power(power).await.is_ok() {
                log::info!("succeeded to set plugs power state after {retry} retries");
                return;
            }
        }

        log::warn!("failed to set plugs power state after max retries");
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
