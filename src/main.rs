mod constants;
mod timer;
mod web;

use tokio::sync::Mutex;
use std::{thread, sync::Arc};

use timer::{Time, YearTimer};
use constants::CHECK_INTERVAL;

#[tokio::main]
async fn main() {

    let year_timer: Arc<Mutex<Option<YearTimer>>> = Arc::new(Mutex::new(None));

    // to avoid matching timers more than once per minute
    let mut last_checked_time = Time::now() - Time::new(0, 1);

    // start webserver ("fire and forget" instead of "await")
    tokio::spawn(web::start_server(Arc::clone(&year_timer)));

    loop {
        let now = Time::now();
        // if timers have already been checked this minute
        if now == last_checked_time {
            return;
        }

        // TODO check if timer matches now 

        last_checked_time = now;

        thread::sleep(CHECK_INTERVAL);
    }
}
