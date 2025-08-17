use std::{sync::Arc, time::Duration};
use tokio::{sync::RwLock, time::{self}};
use crate::{player::Player, traits::{save::DoesSave, Description}, world::SharedWorld};

const LOGOUT_QUEUE_INTERVAL: u64 = 1; // once per second, about.
const AUTOSAVE_QUEUE_INTERVAL: u64 = 300; // once per 5 minutes, about.

/// One heart of the machinery — I/O loop.
/// 
/// # Arguments
/// - `world`— shared world, shared pain ;-)
pub async fn io_loop(world: SharedWorld) {
    let mut logout_interval = time::interval(Duration::from_secs(LOGOUT_QUEUE_INTERVAL));
    let mut autosave_interval = time::interval(Duration::from_secs(AUTOSAVE_QUEUE_INTERVAL));

    log::info!("io_loop firing up …");
    loop {
        tokio::select! {
            _ = logout_interval.tick() => {
                let players_to_save = {
                    let mut w = world.write().await;
                    w.players_to_logout.drain(..).collect::<Vec<Arc<RwLock<Player>>>>()
                };

                if !players_to_save.is_empty() {
                    log::info!("Saving {} disconnected player{}…", players_to_save.len(), if players_to_save.len() == 1{""} else {"s"});
                    for p_arc in players_to_save {
                        let (p_id, r_id) = {
                            let p = p_arc.read().await;
                            (p.id().to_string(), p.location.clone())
                        };

                        {
                            let w = world.write().await;
                            if let Some(r) = w.rooms.get(&r_id) {
                                r.write().await.players.remove(&p_id);
                                log::debug!("Removed disconnected player '{}' from '{}'", &p_id, r_id);
                            }
                        }

                        if let Err(e) = p_arc.write().await.save().await {
                            log::error!("Failed to save player '{}': {:?}", p_id, e);
                        }
                    }
                    log::info!("Disconnected player save cycle complete.");
                }
            },

            _ = autosave_interval.tick() => {
                log::info!("Auto-save cycle initiated …");
                let players = world.read().await.players.clone();
                if !players.is_empty() {
                    log::debug!("Players collected …");
                }
                let mut saved = false;
                for (_, p) in players {
                    let mut p = p.write().await;
                    if let Err(e) = p.save().await {
                        log::error!("Failed to save player '{}': {:?}", p.id(), e);
                    } else {
                        saved = true;
                    }
                }
                if saved {
                    log::info!("Auto-save cycle complete.");
                } else {
                    log::info!("Nothing to do.");
                }
            }
        }
    }
}
