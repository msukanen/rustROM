//! "Make room!" - the [Room] live here.
use std::{collections::HashMap, sync::{Arc, Weak}};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{player::Player, traits::Description, util::direction::Direction, world::area::Area, DATA_PATH};

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Room {
    pub(crate) id: String,
    title: String,
    pub(crate) description: String,
    pub exits: HashMap<Direction, String>,
    #[serde(skip)]
    pub parent: Weak<RwLock<Area>>,
    #[serde(skip, default)]
    pub players: HashMap<String, Weak<RwLock<Player>>>,
}

impl Room {
    /// Bootstrap - staging area rooms #1 and #2.
    pub async fn bootstrap() -> Result<(), std::io::Error> {
        let stem1 = "root";
        let stem2 = "not-so-root";

        // 1st room - the very "root" of all.
        log::warn!("Bootstrap - generating starter room '{}/{}.room'…", *ROOM_PATH, stem1);
        tokio::fs::create_dir_all((*ROOM_PATH).as_str()).await?;
        let room = serde_json::json!({
            "name": "root",
            "title": "The Void",
            "description": "A vast, empty space. It feels like the beginning of something…",
            "exits": {
                "East": "not-so-root"
            }
        });
        tokio::fs::write(
            format!("{}/{}.room", *ROOM_PATH, stem1),
            serde_json::to_string_pretty(&room)?
        ).await?;

        // 2nd room - so that there's somewhere to go from 1st.
        log::warn!("Bootstrap - generating 2nd starter room '{}/{}.room'…", *ROOM_PATH, stem2);
        let room = serde_json::json!({
            "name": "not-so-root",
            "title": "The Void mk.2",
            "description": "A vast, empty space, adjacent to the root emptiness …",
            "exits": {
                "West": "root"
            }
        });
        tokio::fs::write(
            format!("{}/{}.room", *ROOM_PATH, stem2),
            serde_json::to_string_pretty(&room)?
        ).await?;

        log::info!("Bootstrap({}.room, {}.room) OK.", stem1, stem2);
        Ok(())
    }

    /// Get an entirely blank slate.
    pub(crate) fn blank(id: Option<&str>) -> Self { Self {
        id: (if id.is_some() { id.unwrap() } else {""}).into(),
        title: "".into(),
        description: "".into(),
        exits: HashMap::new(),
        parent: Weak::new(),
        players: HashMap::new(),
    }}

    /// Add a [Player] to the [Room].
    /// 
    /// # Arguments
    /// - `player`— Some [Player].
    /// 
    /// # Returns
    /// `true` if player was *really* transferred into the room from elsewhere.
    pub async fn add_player(&mut self, player: &Arc<RwLock<Player>>) -> bool {
        let id: String = player.read().await.id().into();
        if self.players.contains_key(&id) {
            // already present, nothing to do.
            return false;
        }
        // FYI: it's irrelevant if something was replaced or not and thus we ignore .insert() result.
        self.players.insert(id.clone(), Arc::downgrade(&player));
        log::debug!("Player '{}' added to room '{}'", id, self.id());
        return true;
    }

    /// Remove [Player] from the [Room].
    /// 
    /// Note that it is *not* considered an error to try remove [Player] which is
    /// not in this particular room. We'll just silently ignore the call.
    /// 
    /// # Arguments
    /// - `player`— Some [Player].
    pub async fn remove_player(&mut self, player: &Arc<RwLock<Player>>) {
        let id = player.read().await.id().to_string();
        if let Some(_) = self.players.remove(&id) {
            log::debug!("Player '{}' removed from room '{}'", id, self.id());
        }
    }
}

impl Description for Room {
    fn description(&self) -> &str { &self.description }
    fn title(&self) -> &str { &self.title }
    fn id(&self) -> &str { &self.id }
}
