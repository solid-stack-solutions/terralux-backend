mod timer;
mod constants;
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

    let year_timer = Arc::new(Mutex::new(None));

    // to avoid matching timers more than once per minute
    let mut last_checked_time = Time::now() - Time::new(0, 1);

    // start webserver ("fire and forget" instead of "await")
    tokio::spawn(web::start_server(Arc::clone(&year_timer)));

    loop {
        log::trace!("checking for new minute");

        let now = Time::now();
        if now != last_checked_time {
            log::trace!("new minute detected");

            let year_timer = *year_timer.lock().await;
            if let Some(year_timer) = year_timer {
                let day_timer = year_timer.for_today();
                if now == *day_timer.on_time() {
                    log::info!("matched timer, turning plug on");
                } else if now == *day_timer.off_time() {
                    log::info!("matched timer, turning plug off");
                } else {
                    log::trace!("no timer matched");
                }
            } else {
                log::trace!("nothing to check, no timers configured");
            }

            last_checked_time = now;
        }

        tokio::time::sleep(CHECK_INTERVAL).await;
    }
}
