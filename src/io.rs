use std::{collections::HashMap, sync::Arc, time::Duration};
use futures::{stream, StreamExt};
use tokio::{sync::RwLock, time::{self}};
use crate::{item::ItemError, player::Player, string::WordSet, traits::{save::DoesSave, Identity}, util::badname::{load_bad_names, BAD_NAMES_FILEPATH}, world::SharedWorld, AUTOSAVE_QUEUE_INTERVAL};

const LOGOUT_QUEUE_INTERVAL: u64 = 1; // once per second, about.
pub(crate) const DEFAULT_AUTOSAVE_QUEUE_INTERVAL: u64 = 300; // once per 5 minutes, about.
pub(crate) const DEFAULT_AUTOSAVE_ACT_COUNT_THRESHOLD: usize = 16;
#[cfg(not(feature = "localtest"))]
const LOST_AND_FOUND_QUEUE_INTERVAL: u64 = 900; // once per 15 minutes, about.
#[cfg(feature = "localtest")]
const LOST_AND_FOUND_QUEUE_INTERVAL: u64 = 5; // once per 5 sec, about.

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
    let mut config_change_interval = time::interval(Duration::from_millis(100));
    let mut lost_and_found_interval = time::interval(Duration::from_secs(LOST_AND_FOUND_QUEUE_INTERVAL));
    let mut lost_and_found: HashMap<String, ItemError> = HashMap::new();

    log::info!("io_loop firing up … {} second{} logout queue, {} second{} auto-save queue.",
            LOGOUT_QUEUE_INTERVAL, if LOGOUT_QUEUE_INTERVAL==1 {""} else {"s"},
            autosave_interval.period().as_secs(), if autosave_interval.period().as_secs()==1 {""} else {"s"}
        );

    {
        log::info!("Initializing bad words...");
        let mut bwl = bad_words.write().await;
        bwl.clone_from(&load_bad_names(&BAD_NAMES_FILEPATH).await);
        log::info!("Bad words ready → releasing lock for public use.");
    }

    loop {
        tokio::select! {
            _ = config_change_interval.tick() => {
                let new_autosave_interval = Duration::from_secs(*AUTOSAVE_QUEUE_INTERVAL.read().await);
                if new_autosave_interval != autosave_interval.period() {
                    autosave_interval = time::interval(new_autosave_interval);
                }
            },

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
                log::trace!("Auto-save cycle initiated … @{}s intervals.", autosave_interval.period().as_secs());
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
            },

            _ = lost_and_found_interval.tick() => {
                let new_losts = {
                    let mut w = world.write().await;
                    w.lost_and_found.drain().collect::<Vec<(String, ItemError)>>()
                };
                let num = new_losts.len();
                for (id, err) in new_losts {
                    lost_and_found.insert(id, err);
                }
                if num > 0 {
                    log::info!("Collected {} item{} into safety from The Void.", num, if num != 1 {"s"} else {""});
                    log::info!("Currently holding onto {} item{} in total.", lost_and_found.len(), if lost_and_found.len() != 1 {"s"} else {""});
                }
            },
        }
    }
}
