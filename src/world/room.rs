use serde::{Deserialize, Serialize};

pub(crate) mod area_room_serialization {
    use std::fs;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use crate::world::area::AREA_PATH;

    use super::Room;

    pub fn serialize<S: Serializer>(rooms: &Option<Vec<Room>>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        if let Some(rooms) = rooms {
            let room_stems: Vec<String> = rooms.iter().map(|room| {
                let file_path = format!("{}/{}.room", *AREA_PATH, room.name);
                room.name.clone()
            }).collect();
            room_stems.serialize(serializer)
        } else {
            serializer.serialize_unit()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<Room>>, D::Error>
    where D: Deserializer<'de>,
    {
        let room_stems = Vec::<String>::deserialize(deserializer)?;
        let mut loaded_rooms = vec![];

        for stem in room_stems {
            let file_path = format!("{}/{}.room", *AREA_PATH, stem);
            let content = fs::read_to_string(file_path).map_err(serde::de::Error::custom)?;
            let room: Room = serde_json::from_str(&content).map_err(serde::de::Error::custom)?;
            loaded_rooms.push(room);
        }

        Ok(Some(loaded_rooms))
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
