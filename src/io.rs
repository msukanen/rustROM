use std::{sync::Arc, time::Duration};
use futures::{stream, StreamExt};
use tokio::{sync::RwLock, time::{self}};
use crate::{player::Player, string::WordSet, traits::{save::DoesSave, Identity}, util::badname::{load_bad_names, BAD_NAMES_FILEPATH}, world::SharedWorld, AUTOSAVE_QUEUE_INTERVAL};

const LOGOUT_QUEUE_INTERVAL: u64 = 1; // once per second, about.
#[cfg(feature = "localtest")]
pub(crate) const DEFAULT_AUTOSAVE_QUEUE_INTERVAL: u64 = 30; // once per 30 sec, about.
#[cfg(not(feature = "localtest"))]
pub(crate) const DEFAULT_AUTOSAVE_QUEUE_INTERVAL: u64 = 300; // once per 5 minutes, about.
pub(crate) const DEFAULT_AUTOSAVE_ACT_COUNT_THRESHOLD: usize = 16;

/// One heart of the machinery — I/O loop.
/// 
/// # Arguments
/// - `world`— shared world, shared pain ;-)
pub async fn io_loop(
    world: SharedWorld,
    bad_words: Arc<RwLock<WordSet>>,
) {
    let mut logout_interval = time::interval(Duration::from_secs(LOGOUT_QUEUE_INTERVAL));
    let mut autosave_interval = time::interval(Duration::from_secs(*AUTOSAVE_QUEUE_INTERVAL.read().await));

    log::info!("io_loop firing up … {} second{} logout queue, {} second{} auto-save queue.",
            LOGOUT_QUEUE_INTERVAL, if LOGOUT_QUEUE_INTERVAL==1 {""} else {"s"},
            DEFAULT_AUTOSAVE_QUEUE_INTERVAL, if DEFAULT_AUTOSAVE_QUEUE_INTERVAL==1 {""} else {"s"}
        );

    {
        log::info!("Initializing bad words...");
        let mut bwl = bad_words.write().await;
        bwl.clone_from(&load_bad_names(&BAD_NAMES_FILEPATH).await);
        log::info!("Bad words ready → releasing lock for public use.");
    }

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
                log::trace!("Auto-save cycle initiated …");
                let players = world.read().await.players_by_sockaddr.clone();
                let players = stream::iter(players);
                let players = players.filter(|(_,p)|{ let p = p.clone(); async move { p.read().await.act_count() >= DEFAULT_AUTOSAVE_ACT_COUNT_THRESHOLD }});
                let mut saved_any = false;
                players.for_each(|(_,p)| { saved_any = true; async move {
                    let mut w = p.write().await;
                    if let Err(e) = w.save().await {
                        log::error!("Failed to save player '{}': {:?}", w.id(), e);
                    }
                }}).await;
                if saved_any {
                    log::info!("Auto-save cycle complete.");
                } else {
                    log::trace!("… but nothing to do.");
                }
            }
        }
    }
}
