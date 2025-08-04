//! The World
//! 
//! Note about areas and rooms that the world HAS TO HAVE:
//! - an area called `root` (`data/areas/root.area`).
//! - a room called `root` (`data/areas/root.room`).
//! 
//! The dual `root:root` is used as an entrance for new players,
//! guests, and as a fallback after major world changes which
//! cause e.g. saved locations in player saves to be invalid.
//! 
//! If one or the other file is missing… Bad Things™ will happen!
use std::{collections::HashMap, fs::read_to_string, path::PathBuf, str::FromStr, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{cmd::CommandCtx, string::prompt::PromptType, traits::tickable::Tickable, util::contact::{AdminInfo, Contact}, world::{area::{world_area_serialization, Area}, room::Room}, DATA_PATH};

#[derive(Debug, Deserialize, Serialize)]
pub struct MotD {
    text: String,
}

/// World entrance.
/// 
/// Used for locating e.g. players, mobs, rooms themselves, etc.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorldEntrance {
    pub area: String,
    pub room: String,
}

impl WorldEntrance {
    /// Generate default entrance entry.
    pub fn default() -> Self {
        Self {
            area: "root".to_string(),
            room: "root".to_string()
        }
    }
}

/// The World itself…
#[derive(Debug, Deserialize, Serialize)]
pub struct World {
    #[serde(default)]
    uptime: u64,
    #[serde(skip)]
    filename: String,
    title: String,
    description: String,
    owner: Contact,
    admins: Option<Vec<AdminInfo>>,
    pub motd: Option<Vec<MotD>>,
    pub greeting: Option<String>,
    pub welcome_back: Option<String>,
    pub welcome_new: Option<String>,
    #[serde(with = "world_area_serialization")]
    pub areas: HashMap<String, Arc<RwLock<Area>>>,
    pub root: WorldEntrance,
    pub prompts: HashMap<PromptType, String>
}

/// Thread-shared world type.
pub type SharedWorld = Arc<RwLock<World>>;

#[derive(Debug)]
pub enum WorldError {
    Io(std::io::Error),
    Format(serde_json::Error),
}

impl From<std::io::Error> for WorldError {
    fn from(value: std::io::Error) -> Self { Self::Io(value)}
}

impl From<serde_json::Error> for WorldError {
    fn from(value: serde_json::Error) -> Self { Self::Format(value)}
}

impl World {
    /// A brand new world (loaded from a file, of course).
    /// 
    /// # Arguments
    /// - `name`— stem-name of the world. Designates the storage medium (without filename extension, etc.).
    pub fn new(name: &str) -> Result<Self, WorldError> {
        let filename = format!("{}/{}.world", *DATA_PATH, name);
        log::info!("Loading '{}'", filename);
        let path = PathBuf::from_str(filename.as_str()).unwrap();
        let content = read_to_string(path)?;
        let world: World = serde_json::from_str(&content)?;
        Ok(world)
    }

    #[cfg(test)]
    pub async fn do_busy_stuff(&self) {
        use std::time::Duration;

        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    /// Validate the integrity of the loaded world data.
    #[must_use = "This result must be checked to ensure world integrity."]
    pub async fn validate(self) -> Result<Self, String> {
        let mut r: Option<Result<Self, String>> = None;
        // See that the root area and room actually exist.
        if let Some(area) = self.areas.get(&self.root.area) {
            if area.read().await.rooms.get(&self.root.room).is_none() {
                r = Some(Err(format!(
                    "Validation error: root room '{}' defined in '{}' does not exist.",
                    self.root.room, self.filename
                )));
            }
        } else {
            r = Some(Err(format!(
                "Validation error: root area '{}' defined in '{}' does not exist.",
                self.root.area, self.filename
            )))
        }
        r.or(Some(Ok(self))).unwrap()
    }

    /// Transfer to a valid room if 'room' is None.
    /// 
    /// # Returns
    /// `true` if transfer actually had to be done.
    pub fn transfer_to_safety(&self, ctx: &mut CommandCtx<'_>, room: &Option<Arc<Room>>) -> bool {
        if room.is_some() {false}
        else {
            ctx.player.location.area = "root".to_string();
            ctx.player.location.room = "root".to_string();
            true
        }
    }
}

#[async_trait]
impl Tickable for World {
    async fn tick(&mut self, uptime: u64) {
        self.uptime = uptime;
        for area in self.areas.values() {
            area.write().await
                .tick(self.uptime).await;
        }
    }
}

#[cfg(test)]
mod world_tests {
    use super::*;
    use std::time::Duration;
    use log::debug;
    use crate::game_loop::game_loop;

    /// Let's see how the threads react to the core world being super busy with global locks.
    #[tokio::test]
    async fn busy_world() {
        let _ = env_logger::try_init();
        let world = Arc::new(RwLock::new(World::new("rustrom").expect("ERROR: world dead or in fire?!")));

        tokio::spawn(game_loop(world.clone()));
        {
            tokio::time::sleep(Duration::from_secs(2)).await;
            debug!("Enter guard...");
            {
                let w = world.write().await;
                w.do_busy_stuff().await;
                debug!("Exit guard...");
            }
            debug!("Waited busy stuff...");
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
        debug!("Lazing about.");
    }
}
