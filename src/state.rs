use tokio::sync::Mutex;
use std::sync::Arc;
use chrono_tz::Tz;

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
    /// timers to check every day
    pub year_timer: year::Timer,
}
