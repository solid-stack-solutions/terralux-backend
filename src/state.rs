use tokio::sync::Mutex;
use std::sync::Arc;

use crate::plug::Plug;
use crate::timer::year;

#[allow(clippy::module_name_repetitions)]
pub type StateWrapper = Arc<Mutex<Option<State>>>;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct State {
    pub plug: Plug,
    pub year_timer: year::Timer,
}
