//! Area stuff.
use std::{collections::HashMap, sync::{Arc, Weak}};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{traits::tickable::Tickable, world::{room::{area_room_serialization, Room}, World}, DATA_PATH};

static AREA_PATH: Lazy<Arc<String>> = Lazy::new(|| Arc::new(format!("{}/areas", *DATA_PATH)));

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

#[derive(Debug, Deserialize, Serialize)]
pub struct Area {
    pub name: String,
    description: String,
    #[serde(with = "area_room_serialization")]
    pub rooms: HashMap<String, Arc<RwLock<Room>>>,
    #[serde(skip)]
    pub parent: Weak<RwLock<World>>,
}

#[async_trait]
impl Tickable for Area {
    async fn tick(&mut self, uptime: u64) {
        
    }
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
        debug!("Area name: \"{}\"", area.name);
        debug!("     desc: \"{}\"", area.description);
        
    }
}
