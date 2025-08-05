use std::time::Duration;
use tokio::time::{sleep_until, Instant};
use crate::{traits::tickable::Tickable, world::SharedWorld};

const MILLIS_PER_TICK: u64 = 100; // ~10 ticks/sec (with 100ms each).

/// The heart of the machinery — game loop.
/// 
/// # Arguments
/// - `world`— shared world, shared pain ;-)
pub async fn game_loop(world: SharedWorld) {
    let duration = Duration::from_millis(MILLIS_PER_TICK);
    let mut next_tick = Instant::now() + duration;
    let mut uptime = 0;

    log::info!("game_loop firing up …");
    loop {
        sleep_until(next_tick).await;
        next_tick += duration;
        uptime += 1;
        #[cfg(test)]{log::debug!("game_loop - Tick {}", uptime);}

        // Tick-tock goes the clock and the world spins 'round and 'round…
        world.write().await.tick(uptime).await;

        if Instant::now() > next_tick {
            log::warn!("Clock skew! Busy day! Lagging behind!");
            next_tick = Instant::now() + duration;
        }
    }
}
