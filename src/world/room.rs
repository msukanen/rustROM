//! "Make room!" - the [Room] live here.
use std::{collections::HashMap, sync::{Arc, Weak}};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{traits::Description, util::direction::Direction, world::area::Area, DATA_PATH};

static ROOM_PATH: Lazy<Arc<String>> = Lazy::new(|| Arc::new(format!("{}/rooms", *DATA_PATH)));

pub enum RoomError {
    NoRoom,
}

/// Room serializer for [Area]-level hashmap.
pub mod area_room_serialization {
    use std::{collections::HashMap, fs, sync::Arc};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use tokio::sync::RwLock;

    use super::{Room, ROOM_PATH};

    pub fn serialize<S: Serializer>(rooms: &HashMap<String, Arc<RwLock<Room>>>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        for (stem, room) in rooms {
            let path = format!("{}/{}.room", *ROOM_PATH, stem);
            let contents = runtime.block_on(async {
                let g = room.read().await;
                serde_json::to_string_pretty(&*g).unwrap()
            });
            fs::write(path, contents).unwrap();
        }

        rooms.keys()
            .collect::<Vec<&String>>()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<String, Arc<RwLock<Room>>>, D::Error>
    where D: Deserializer<'de>,
    {
        let room_stems = Vec::<String>::deserialize(deserializer)?;
        let mut loaded_rooms = HashMap::new();

        for stem in room_stems {
            let path = format!("{}/{}.room", *ROOM_PATH, stem);
            log::info!("… processing '{}'", path);
            let contents = fs::read_to_string(path).map_err(serde::de::Error::custom)?;
            let room: Room = serde_json::from_str(&contents).map_err(serde::de::Error::custom)?;
            loaded_rooms.insert(stem.to_string(), Arc::new(RwLock::new(room)));
        }

        Ok(loaded_rooms)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Room {
    name: String,
    title: String,
    description: String,
    pub exits: HashMap<Direction, String>,
    #[serde(skip)]
    pub parent: Weak<RwLock<Area>>,
}

impl Room {
    pub async fn bootstrap() -> Result<(), std::io::Error> {
        log::warn!("Bootstrap - generating starter room '{}/root.room'…", *ROOM_PATH);
        tokio::fs::create_dir_all((*ROOM_PATH).as_str()).await?;
        let room = serde_json::json!({
            "name": "root",
            "title": "The Void",
            "description": "A vast, empty space. It feels like the beginning of something…",
            "exits": {}
        });
        tokio::fs::write(format!("{}/root.room", *ROOM_PATH), serde_json::to_string_pretty(&room)?).await?;
        log::info!("Bootstrap(root.room) OK.");
        Ok(())
    }
}

impl Description for Room {
    fn description(&self) -> &str { &self.description }
    fn title(&self) -> &str { &self.title }
    fn name(&self) -> &str { &self.name }
}
