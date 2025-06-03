mod timer;
mod constants;
mod sunrise_api;
mod time;
mod web;

use tokio::sync::Mutex;
use std::sync::Arc;

use time::Time;
use timer::year;
use sunrise_api::SunriseAPI;
use constants::CHECK_INTERVAL;

#[tokio::main]
async fn main() {
    // set up logging with default level if env var `RUST_LOG` is unset
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(
            format!("error,{}=info", env!("CARGO_PKG_NAME"))
    )).init();

    let year_timer = {
        // local: bremen, germany
        log::info!("requesting local data from API");
        let local_api_days = SunriseAPI::new().request(53.1, 8.8).await.unwrap();

        // avoid API rate limiting
        tokio::time::sleep(std::time::Duration::from_millis(430)).await;

        // natural: new caledonia
        log::info!("requesting natural data from API");
        let natural_api_days = SunriseAPI::new().request(-21.3, 165.4).await.unwrap();

        log::info!("averaging data");
        year::Timer::from_api_days_average(0.5, &local_api_days, &natural_api_days)
    };
    log::info!("determined year timer");

    let year_timer = Arc::new(Mutex::new(Some(year_timer)));

    // to avoid matching timers more than once per minute
    let mut last_checked_time = Time::now() - Time::new(0, 1);

    // start webserver ("fire and forget" instead of "await")
    tokio::spawn(web::start_server(Arc::clone(&year_timer)));

    loop {
        let now = Time::now();
        if now != last_checked_time {

            let year_timer = *year_timer.lock().await;
            if let Some(year_timer) = year_timer {
                let day_timer = year_timer.for_today();
                if now == *day_timer.on_time() {
                    log::info!("matched timer, turning plug on");
                }
                if now == *day_timer.off_time() {
                    log::info!("matched timer, turning plug off");
                }
            }

            last_checked_time = now;
        }

        tokio::time::sleep(CHECK_INTERVAL).await;
    }
}
