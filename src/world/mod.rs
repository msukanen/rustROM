use std::{collections::HashMap, fs::read_to_string, path::PathBuf, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{string::prompt::PromptType, traits::tickable::Tickable, util::contact::{AdminInfo, Contact}, world::area::{Area, world_area_serialization}, DATA_PATH};
pub(crate) mod area;
pub(crate) mod room;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct MotD {
    text: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorldEntrance {
    area: String,
    room: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct World {
    #[serde(default)]
    uptime: u64,
    title: String,
    description: String,
    owner: Contact,
    admins: Option<Vec<AdminInfo>>,
    pub motd: Option<Vec<MotD>>,
    pub greeting: Option<String>,
    pub welcome_back: Option<String>,
    pub welcome_new: Option<String>,
    #[serde(with = "world_area_serialization")]
    areas: Option<Vec<Area>>,
    root: Option<WorldEntrance>,
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
}

impl Tickable for World {
    fn tick(&mut self, uptime: u64) {
        self.uptime = uptime;
        if let Some(areas) = &mut self.areas {
            for area in areas.iter_mut() {
                area.tick(self.uptime);
            }
        }
    }
}

#[cfg(test)]
mod world_tests {
    use std::time::Duration;

    use log::debug;

    use crate::game_loop::game_loop;

    use super::*;

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
