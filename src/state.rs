use tokio::sync::Mutex;
use std::sync::Arc;
use chrono_tz::Tz;

use crate::time::Time;
use crate::plug::Plug;
use crate::timer::year;

#[allow(clippy::module_name_repetitions)]
pub type StateWrapper = Arc<Mutex<Option<State>>>;

/// state of the application (also saved/loaded from/to json file)
#[derive(serde::Serialize, serde::Deserialize)]
pub struct State {
    /// average sunrise/sunset times between local ones (0.0) and ones from the natural habitat (1.0)
    pub natural_factor: f32,
    /// latitude of geographic coordinates of terrarium, from -90° (south) to 90° (north)
    pub local_latitude: f32,
    /// longitude of geographic coordinates of terrarium, from -180° (west) to 180° (east)
    pub local_longitude: f32,
    /// latitude of geographic coordinates of the animals natural habitat, from -90° (south) to 90° (north)
    pub natural_latitude: f32,
    /// longitude of geographic coordinates of the animals natural habitat, from -180° (west) to 180° (east)
    pub natural_longitude: f32,
    /// plug to control
    pub plug: Plug,
    /// timezone to use for timer activations
    pub timezone: Tz,
    /// actual timers to turn plug on/off every day
    pub year_timer: year::Timer,
}

impl State {
    pub fn read_from_file() -> Option<Self> {
        let path = dirs_next::data_dir();
        if path.is_none() {
            log::debug!("couldn't get path to data directory, operating system probably unsupported");
            return None;
        };

        let mut path = path.unwrap();
        path.push(crate::constants::STATE_FILE_NAME);

        let content = std::fs::read_to_string(path);
        if content.is_err() {
            log::info!("no state file found, waiting for configuration");
            return None;
        };

        let state = serde_json::from_str::<Self>(&content.unwrap());
        if state.is_err() {
            log::warn!("read state file, but content did not have the expected structure");
            return None;
        };

        log::info!("successfully read last state from file");

        let state = state.unwrap();
        let timezone = state.timezone;
        log::info!("using timezone {timezone}, current time is {}", Time::now(timezone));

        Some(state)
    }

    pub fn write_to_file(state: StateWrapper) {
        // try to write file as a "fire and forget" as its result does not need to be awaited
        tokio::spawn(async move {
            let path = dirs_next::data_dir();
            if path.is_none() {
                log::warn!("couldn't get path to data directory to write state file to, your operating system is unsupported");
                return;
            };

            let mut path = path.unwrap();
            path.push(crate::constants::STATE_FILE_NAME);

            let content = serde_json::to_string(&*state.lock().await).unwrap();

            match tokio::fs::write(path, content).await {
                Ok(()) => log::info!("successfully wrote state file"),
                Err(_) => log::warn!("failed to write state file"),
            };
        });
    }
}
