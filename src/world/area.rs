//! Area stuff.
use std::{collections::HashMap, sync::{Arc, Weak}};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{traits::{tickable::Tickable, Description}, world::{room::{area_room_serialization, Room}, World}, DATA_PATH};

static AREA_PATH: Lazy<Arc<String>> = Lazy::new(|| Arc::new(format!("{}/areas", *DATA_PATH)));
const DEFAULT_TICK_MODULO: u8 = 10;// normally an Area acts once every 10th tick.

pub mod world_area_serialization {
    //! Serializer for [World] level [Area] listing.
    use std::{collections::HashMap, fs, sync::Arc};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use tokio::sync::RwLock;

    use super::{Area, AREA_PATH};

    pub fn serialize<S: Serializer>(areas: &HashMap<String, Arc<RwLock<Area>>>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        for (stem, area) in areas {
            let path = format!("{}/{}.area", *AREA_PATH, stem);
            log::info!("Processing '{}'", path);
            let contents = runtime.block_on(async {
                let g = area.read().await;
                serde_json::to_string_pretty(&*g).unwrap()
            });
            fs::write(path, contents).unwrap();
        }
        areas.keys()
            .collect::<Vec<&String>>()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<String, Arc<RwLock<Area>>>, D::Error>
    where D: Deserializer<'de>,
    {
        let stems = Vec::<String>::deserialize(deserializer)?;
        let mut loaded = HashMap::new();

        for stem in stems {
            let path = format!("{}/{}.area", *AREA_PATH, stem);
            log::info!("… processing '{}'", path);
            let contents = fs::read_to_string(path).map_err(serde::de::Error::custom)?;
            let area: Area = serde_json::from_str(&contents).map_err(serde::de::Error::custom)?;
            loaded.insert(stem, Arc::new(RwLock::new(area)));
        }

        Ok(loaded)
    }
}

fn default_tick_modulo() -> u8 {DEFAULT_TICK_MODULO}// to appease 'serde(default = ...)'
#[derive(Debug, Deserialize, Serialize)]
pub struct Area {
    pub id: String,
    title: String,
    description: String,
    #[serde(with = "area_room_serialization")]
    pub rooms: HashMap<String, Arc<RwLock<Room>>>,
    #[serde(skip)]
    pub parent: Weak<RwLock<World>>,
    #[serde(default = "default_tick_modulo")]
    tick_modulo: u8,
}

#[async_trait]
impl Tickable for Area {
    async fn tick(&mut self, uptime: u64) {
        // Time to tick?
        if (uptime % self.tick_modulo as u64) != 0 {return ;}
    }
}

impl Area {
    /// Bootstrap - staging area.
    pub async fn bootstrap() -> Result<(), std::io::Error> {
        let stem = "root";
        log::warn!("Bootstrap - generating starter area '{}/{}.area'…", *AREA_PATH, stem);
        tokio::fs::create_dir_all((*AREA_PATH).as_str()).await?;
        let area = serde_json::json!({
            "name": "root",
            "title": "The Genesis Area",
            "description": "Where it all begins …",
            "rooms": ["root", "not-so-root"]
        });
        tokio::fs::write(format!("{}/{}.area", *AREA_PATH, stem), serde_json::to_string_pretty(&area)?).await?;
        log::info!("Bootstrap({}.area) OK.", stem);
        Ok(())
    }

    /// A blank slate.
    pub(crate) fn blank() -> Self { Self {
        id: "".into(),
        title: "".into(),
        description: "".into(),
        rooms: HashMap::new(),
        parent: Weak::new(),
        tick_modulo: 10
    }}
}

impl Description for Area {
    fn description(&self) -> &str { &self.description }
    fn id(&self) -> &str { &self.id }
    fn title(&self) -> &str { &self.title }
}

#[cfg(test)]
mod area_tests {
    use std::path::PathBuf;

    use log::debug;

    use super::*;

    #[test]
    fn load_area() {
        let _ = env_logger::try_init();
        let contents = std::fs::read_to_string(PathBuf::from(format!("{}/root.area", *AREA_PATH))).expect("Cannot find?!");
        debug!("Con10z:\n{}", contents);
        let area: Area = serde_json::from_str(&contents).unwrap();
        debug!("Area name: \"{}\"", area.id);
        debug!("     desc: \"{}\"", area.description);
        
    }
}
