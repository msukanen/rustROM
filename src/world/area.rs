//! Area stuff.
use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{traits::tickable::Tickable, world::room::{area_room_serialization, Room}, DATA_PATH};

static AREA_PATH: Lazy<Arc<String>> = Lazy::new(|| Arc::new(format!("{}/areas", *DATA_PATH)));

pub(crate) mod world_area_serialization {
    //! Serializer for [World] level [Area] listing.
    use std::{collections::HashMap, fs};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{Area, AREA_PATH};

    pub fn serialize<S: Serializer>(areas: &HashMap<String, Area>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        for (stem, area) in areas {
            let area_path = format!("{}/{}.area", *AREA_PATH, stem);
            let area_content = serde_json::to_string_pretty(area).unwrap();
            fs::write(area_path, area_content).unwrap();
        }
        areas.keys()
            .collect::<Vec<&String>>()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<String, Area>, D::Error>
    where D: Deserializer<'de>,
    {
        let stems = Vec::<String>::deserialize(deserializer)?;
        let mut loaded = HashMap::new();

        for stem in stems {
            let file_path = format!("{}/{}.area", *AREA_PATH, stem);
            let content = fs::read_to_string(file_path).map_err(serde::de::Error::custom)?;
            let content: Area = serde_json::from_str(&content).map_err(serde::de::Error::custom)?;
            loaded.insert(stem, content);
        }

        Ok(loaded)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Area {
    pub(crate) name: String,
    description: String,
    #[serde(with = "area_room_serialization")]
    rooms: HashMap<String, Room>,
}

impl Tickable for Area {
    fn tick(&mut self, uptime: u64) {
        
    }
}

impl Area {
    pub(crate) fn find_room(&self, name: &str) -> Option<&Room> {
        self.rooms.get(name)
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
