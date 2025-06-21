use crate::state::{State, StateWrapper};
use crate::time::Time;

pub fn read() -> Option<State> {
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

    let state = serde_json::from_str::<State>(&content.unwrap());
    if state.is_err() {
        log::warn!("read state file, but content did not have the expected structure");
        return None;
    };

    log::info!("successfully read last state from file");

    let state = state.unwrap();
    let timezone = *state.year_timer.timezone();
    log::info!("using timezone {timezone}, current time is {}", Time::now(timezone));

    Some(state)
}

#[allow(clippy::significant_drop_tightening)]
pub fn write(state: StateWrapper) {
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
