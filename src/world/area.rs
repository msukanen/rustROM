use std::sync::Arc;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{DATA_PATH, traits::tickable::Tickable, world::room::{Room, area_room_serialization}};

pub(crate) static AREA_PATH: Lazy<Arc<String>> = Lazy::new(|| Arc::new(format!("{}/areas", *DATA_PATH)));

pub(crate) mod world_area_serialization {
    use std::fs;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use crate::world::area::AREA_PATH;

    use super::Area;

    pub fn serialize<S: Serializer>(areas: &Option<Vec<Area>>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        if let Some(areas) = areas {
            let area_stems: Vec<String> = areas.iter().map(|area| {
                let file_path = format!("{}/{}.area", *AREA_PATH, area.name);
                area.name.clone()
            }).collect();
            area_stems.serialize(serializer)
        } else {
            serializer.serialize_unit()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<Area>>, D::Error>
    where D: Deserializer<'de>,
    {
        let stems = Vec::<String>::deserialize(deserializer)?;
        let mut loaded = vec![];

        for stem in stems {
            let file_path = format!("{}/{}.area", *AREA_PATH, stem);
            let content = fs::read_to_string(file_path).map_err(serde::de::Error::custom)?;
            let content: Area = serde_json::from_str(&content).map_err(serde::de::Error::custom)?;
            loaded.push(content);
        }

        Ok(Some(loaded))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Area {
    pub(crate) name: String,
    description: String,
    #[serde(with = "area_room_serialization")]
    rooms: Option<Vec<Room>>,
}

impl Tickable for Area {
    fn tick(&mut self, uptime: u64) {
        
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
        let contents = std::fs::read_to_string(PathBuf::from(format!("{}/areas/root.area", std::env::var("RUSTROM_DATA").unwrap()))).expect("Cannot find?!");
        debug!("Con10z:\n{}", contents);
        let area: Area = serde_json::from_str(&contents).unwrap();
        debug!("Area name: \"{}\"", area.name);
        debug!("     desc: \"{}\"", area.description);
        if let Some(rooms) = &area.rooms {
            for room in rooms.into_iter() {
                debug!("{}", room.description());
            }
        }
    }
}
