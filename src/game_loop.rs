use std::time::Duration;

use tokio::time;

use crate::{traits::tickable::Tickable, world::{SharedWorld, World}};

pub(crate) async fn game_loop(world: SharedWorld) {
    let mut interval = time::interval(Duration::from_secs(1));
    let mut uptime = 0;

    loop {
        interval.tick().await;
        uptime += 1;

        {
            let mut world: tokio::sync::MutexGuard<World> = world.lock().await;
            world.tick(uptime);
        }
    }
}
