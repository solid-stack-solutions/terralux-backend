mod api;
mod timer;
mod constants;
mod plug;
mod state;
mod sunrise_api;
mod time;

use tokio::sync::Mutex;
use std::sync::Arc;

use time::Time;
use state::State;
use constants::CHECK_INTERVAL;

#[tokio::main]
async fn main() {
    // set up logging with default level if env var `RUST_LOG` is unset
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(
            format!("error,{}=info", env!("CARGO_PKG_NAME").replace('-', "_"))
    )).init();

    if cfg!(feature = "mock_plug") {
        log::info!("mock_plug feature detected, mocking requests to smart plug");
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "demo_mode")] {
            log::info!("demo_mode feature detected, accelerating flow of time: {}min (real) => 24h (simulated)",
                constants::MINUTES_PER_DAY);
        }
    }

    // thread-safe state (persisted with json file)
    let state = Arc::new(Mutex::new(State::read_from_file()));

    // to avoid matching timers more than once per minute
    let mut last_checked_time = None;

    // start webserver ("fire and forget" instead of "await")
    tokio::spawn(api::start_server(Arc::clone(&state)));

    loop {
        #[allow(clippy::significant_drop_in_scrutinee)]
        if let Some(ref state) = *state.lock().await {
            log::trace!("checking for new minute");

            let now = Time::now(state.timezone);
            if last_checked_time != Some(now) {
                if cfg!(feature = "demo_mode") && now.minute() % 15 == 0 {
                    log::info!("it is {now}");
                } else {
                    log::trace!("new minute detected");
                }

                let day_timer = state.year_timer.for_today(state.timezone);
                if now == *day_timer.on_time() {
                    log::info!("matched timer for {now}, turning plug on");
                    state.plug.set_power_with_retry(true).await;
                } else if now == *day_timer.off_time() {
                    log::info!("matched timer for {now}, turning plug off");
                    state.plug.set_power_with_retry(false).await;
                } else {
                    log::trace!("no timer matched");
                }

                last_checked_time = Some(now);
            }
        } else {
            log::trace!("nothing to check, no timers configured");
        }

        tokio::time::sleep(CHECK_INTERVAL).await;
    }
}
