use crate::plug::Plug;
use crate::timer::year;
use crate::web::{StatePlug, StateYearTimer};

pub fn read() -> Option<(Plug, year::Timer)> {
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

    let state = serde_json::from_str(&content.unwrap());
    if state.is_err() {
        log::warn!("read state file, but content did not have the expected structure");
        return None;
    };

    log::info!("successfully read last state from file");
    Some(state.unwrap())
}

#[allow(clippy::significant_drop_tightening)]
pub fn write(state_plug: StatePlug, state_year_timer: StateYearTimer) {
    // try to write file as a "fire and forget" as its result does not need to be awaited
    tokio::spawn(async move {
        let path = dirs_next::data_dir();
        if path.is_none() {
            log::warn!("couldn't get path to data directory to write state file to, your operating system is unsupported");
            return;
        };

        let mut path = path.unwrap();
        path.push(crate::constants::STATE_FILE_NAME);

        let content = {
            let locked_plug = state_plug.lock().await;
            let locked_year_timer = state_year_timer.lock().await;
            let state = (
                locked_plug.as_ref().unwrap(),
                locked_year_timer.as_ref().unwrap(),
            );
            serde_json::to_string(&state).unwrap()
        };

        match tokio::fs::write(path, content).await {
            Ok(()) => log::info!("successfully wrote state file"),
            Err(_) => log::warn!("failed to write state file"),
        };
    });
}
