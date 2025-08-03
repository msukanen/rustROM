//! "Make room!" - the [Room] live here.
use std::sync::Arc;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::DATA_PATH;

static ROOM_PATH: Lazy<Arc<String>> = Lazy::new(|| Arc::new(format!("{}/rooms", *DATA_PATH)));

/// Room serializer for [Area]-level hashmap.
pub(crate) mod area_room_serialization {
    use std::{collections::HashMap, fs};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{Room, ROOM_PATH};

    pub fn serialize<S: Serializer>(rooms: &HashMap<String, Room>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        for (stem, room) in rooms {
            let path = format!("{}/{}.room", *ROOM_PATH, stem);
            let contents = serde_json::to_string_pretty(room).unwrap();
            fs::write(path, contents).unwrap();
        }

        rooms.keys()
            .collect::<Vec<&String>>()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<String, Room>, D::Error>
    where D: Deserializer<'de>,
    {
        let room_stems = Vec::<String>::deserialize(deserializer)?;
        let mut loaded_rooms: HashMap<String, Room> = HashMap::new();

        for stem in room_stems {
            let path = format!("{}/{}.room", *ROOM_PATH, stem);
            let contents = fs::read_to_string(path).map_err(serde::de::Error::custom)?;
            let room: Room = serde_json::from_str(&contents).map_err(serde::de::Error::custom)?;
            loaded_rooms.insert(stem.to_string(), room);
        }

        Ok(loaded_rooms)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Room {
    pub(crate) name: String,
    title: String,
    description: String,
}

impl Room {
    pub fn description(&self) -> &str {
        &self.description
    }
}
