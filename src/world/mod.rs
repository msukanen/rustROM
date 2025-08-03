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

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{string::prompt::PromptType, traits::tickable::Tickable, util::contact::{AdminInfo, Contact}, world::{area::{world_area_serialization, Area}, room::Room}, DATA_PATH};
pub(crate) mod area;
pub(crate) mod room;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct MotD {
    text: String,
}

/// World entrance.
/// 
/// Used for locating e.g. players, mobs, rooms themselves, etc.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct WorldEntrance {
    pub(crate) area: String,
    pub(crate) room: String,
}

impl WorldEntrance {
    /// Generate default entrance entry.
    pub(crate) fn default() -> Self {
        Self {
            area: "root".to_string(),
            room: "root".to_string()
        }
    }
}

/// The World itself…
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct World {
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
    areas: HashMap<String, Area>,
    root: WorldEntrance,
    pub prompts: HashMap<PromptType, String>
}

/// Thread-shared world type.
pub type SharedWorld = Arc<RwLock<World>>;

#[derive(Debug)]
pub(crate) enum WorldError {
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
    pub(crate) fn new(name: &str) -> Result<Self, WorldError> {
        let filename = format!("{}/{}.world", *DATA_PATH, name);
        let path = PathBuf::from_str(filename.as_str()).unwrap();
        let content = read_to_string(path)?;
        let world: World = serde_json::from_str(&content)?;
        Ok(world)
    }

    #[cfg(test)]
    pub(crate) async fn do_busy_stuff(&self) {
        use std::time::Duration;

        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    /// Finds a specific area of the world.
    pub(crate) fn find_area(&self, name: &str) -> Option<&Area> {
        self.areas.get(name)
    }

    /// Finds a room within a specific area of the world.
    pub(crate) fn find_room(&self, area_name: &str, room_name: &str) -> Option<&Room> {
        self.find_area(area_name)
            .and_then(|area| {
                area.find_room(room_name)
            })
    }

    /// Validate the integrity of the loaded world data.
    #[must_use = "This result must be checked to ensure world integrity."]
    pub(crate) fn validate(self) -> Result<Self, String> {
        // See that the root area and room actually exist.
        if let Some(area) = self.find_area(&self.root.area) {
            if let Some(_) = area.find_room(&self.root.room) {
                // TODO: other sorts of validation?
                Ok(self)
            } else {
                Err(format!(
                    "Validation error: root room '{}' defined in '{}' does not exist.",
                    self.root.room, self.filename
                ))
            }
        } else {
            Err(format!(
                "Validation error: root area '{}' defined in '{}' does not exist.",
                self.root.area, self.filename
            ))
        }
    }
}

impl Tickable for World {
    fn tick(&mut self, uptime: u64) {
        self.uptime = uptime;
        for (_, area) in &mut self.areas.iter_mut() {
            area.tick(self.uptime);
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
