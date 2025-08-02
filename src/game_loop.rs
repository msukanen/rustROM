use std::time::Duration;

use tokio::time::{sleep_until, Instant};

use crate::{traits::tickable::Tickable, world::SharedWorld};

pub(crate) async fn game_loop(world: SharedWorld) {
    let duration = Duration::from_secs(1);
    let mut next_tick = Instant::now() + duration;
    let mut uptime = 0;

    log::info!("game_loop firing up …");
    #[cfg(test)]{log::debug!("game_loop - Tick {}", uptime);}
    loop {
        sleep_until(next_tick).await;
        next_tick += duration;
        uptime += 1;
        #[cfg(test)]{log::debug!("game_loop - Tick {}", uptime);}

        world.write().await.tick(uptime);

        if Instant::now() > next_tick {
            log::warn!("Clock skew! Busy day! Lagging behind!");
            next_tick = Instant::now() + duration;
        }
    }
}
