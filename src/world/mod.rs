use std::{collections::HashMap, convert::Infallible, fs::read_to_string, path::PathBuf, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{string::PromptType, traits::tickable::Tickable, util::contact::{AdminInfo, Contact}, world::area::Area, DATA_PATH};
pub(crate) mod area;
pub(crate) mod room;

#[derive(Debug, Deserialize, Serialize)]
struct MotD {
    text: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorldEntrance {
    area: String,
    room: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct World {
    uptime: u64,
    title: String,
    description: String,
    owner: Contact,
    admins: Option<Vec<AdminInfo>>,
    motd: Option<Vec<MotD>>,
    greeting: Option<String>,
    areas: Option<Vec<Area>>,
    root: Option<WorldEntrance>,
    prompts: HashMap<PromptType, String>
}

pub type SharedWorld = Arc<Mutex<World>>;

#[derive(Debug)]
pub(crate) enum WorldError {
    Path(Infallible),
    Io(std::io::Error),
    Format(serde_json::Error),
}

impl From<Infallible> for WorldError {
    fn from(value: Infallible) -> Self { Self::Path(value)}
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
        let path = PathBuf::from_str(filename.as_str())?;
        let content = read_to_string(path)?;
        let world: World = serde_json::from_str(&content)?;
        Ok(world)
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
