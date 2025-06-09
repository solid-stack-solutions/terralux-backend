mod timer;
mod constants;
mod plug;
mod state_file;
mod sunrise_api;
mod time;
mod web;

use tokio::sync::Mutex;
use std::sync::Arc;

use time::Time;
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

    let (plug, year_timer) = match state_file::read() {
        Some((plug, year_timer)) => (Some(plug), (Some(year_timer))),
        None => (None, None),
    };
    let plug = Arc::new(Mutex::new(plug));
    let year_timer = Arc::new(Mutex::new(year_timer));

    // to avoid matching timers more than once per minute
    let mut last_checked_time = None;

    // start webserver ("fire and forget" instead of "await")
    tokio::spawn(web::start_server(Arc::clone(&year_timer), Arc::clone(&plug)));

    loop {
        let year_timer = *year_timer.lock().await;
        if let Some(ref year_timer) = year_timer {
            log::trace!("checking for new minute");

            let now = Time::now(*year_timer.timezone());
            if last_checked_time.map_or(true, |last_checked_time| now != last_checked_time) {
                if cfg!(feature = "demo_mode") && now.minute() % 15 == 0 {
                    log::info!("it is {}", now);
                } else {
                    log::trace!("new minute detected");
                }

                let day_timer = year_timer.for_today();
                if now == *day_timer.on_time() {
                    log::info!("matched timer for {now}, turning plug on");
                    plug.lock().await.as_ref().unwrap().set_power_with_retry(true).await;
                } else if now == *day_timer.off_time() {
                    log::info!("matched timer for {now}, turning plug off");
                    plug.lock().await.as_ref().unwrap().set_power_with_retry(false).await;
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
