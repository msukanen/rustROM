//! "Make room!" - the [Room] live here.
use std::{collections::{HashMap, HashSet, VecDeque}, sync::{Arc, Weak}};

use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer, Serialize};
use tokio::sync::RwLock;

use crate::{item::{inventory::{Container, ContainerType, Storage, StorageCapacity}, Item, ItemError}, player::Player, traits::{Description, Identity}, util::{direction::Direction, Editor}, world::{area::Area, SharedWorld}, DATA_PATH};

static ROOM_PATH: Lazy<Arc<String>> = Lazy::new(|| Arc::new(format!("{}/rooms", *DATA_PATH)));
/// Max number of items in a [Room], whether on ground or otherwise.
pub(crate) static MAX_ITEMS_IN_ROOM: usize = 1_000;

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
pub enum ExitState {
    Open,
    Closed,
    Locked { key_id: String }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Exit {
    pub destination: String,
    #[serde(default)]
    pub state: ExitState,
}

impl Default for ExitState {
    fn default() -> Self {
        ExitState::Open
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Room {
    pub(crate) id: String,
    title: String,
    pub(crate) description: String,
    pub exits: HashMap<Direction, Exit>,
    #[serde(skip)] pub parent: Weak<RwLock<Area>>,
    #[serde(skip, default)] pub players: HashMap<String, Weak<RwLock<Player>>>,
    pub contents: Container,
}

// We manually implement Deserialize for it
impl<'de> Deserialize<'de> for Room {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RoomData {
            id: String,
            title: String,
            description: String,
            exits: HashMap<Direction, Exit>,
        }

        // 2. Deserialize the data into our simple, temporary struct.
        let data = RoomData::deserialize(deserializer)?;

        // 3. Now, we have the `id`! We can use it to correctly
        //    construct the Container.
        let contents = Container::from(ContainerType::Room(data.id.clone()));

        // 4. Finally, build the real Room object.
        Ok(Room {
            id: data.id,
            title: data.title,
            description: data.description,
            exits: data.exits,
            contents,
            parent: Weak::new(), // Will be linked later
            players: HashMap::new(), // Will be populated at runtime
        })
    }
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
    pub(crate) fn blank(id: Option<&str>) -> Self {
        let id: String = (if id.is_some() { id.unwrap() } else {""}).into();

        Self {
            contents: Container::from(ContainerType::Room(id.clone())),
            id,
            title: "".into(),
            description: "".into(),
            exits: HashMap::new(),
            parent: Weak::new(),
            players: HashMap::new(),
        }
    }

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
}

impl Identity for Room {
    fn id<'a>(&'a self) -> &'a str { &self.id }
}

/// Finds all rooms within a given distance of a starting room using BFS.
pub(crate) async fn find_nearby_rooms(world: &SharedWorld, start_room_id: &str, max_distance: u32) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut nearby = HashSet::new();

    // The queue stores tuples of (room_id, distance)
    queue.push_back((start_room_id.to_string(), 0));
    visited.insert(start_room_id.to_string());

    let world_guard = world.read().await;

    while let Some((current_room_id, distance)) = queue.pop_front() {
        if distance > max_distance {
            continue;
        }
        nearby.insert(current_room_id.clone());

        if distance < max_distance {
            if let Some(current_room_arc) = world_guard.rooms.get(&current_room_id) {
                let current_room = current_room_arc.read().await;
                for dest_exit in current_room.exits.values() {
                    if !visited.contains(&dest_exit.destination) {
                        visited.insert(dest_exit.destination.clone());
                        queue.push_back((dest_exit.destination.clone(), distance + 1));
                    }
                }
            }
        }
    }
    
    nearby
}

impl Editor for Room {
    fn set_description(&mut self, desc: &str) {
        log::debug!("Setting description as: {}", desc);
        self.description = desc.into();
    }
}

impl StorageCapacity for Room {
    fn capacity(&self) -> usize {
        self.contents.capacity()
    }

    fn num_items(&self) -> usize {
        self.contents.num_items()
    }

    fn space(&self) -> usize {
        self.contents.space()
    }
}

impl Storage for Room {
    fn try_insert(&mut self, item: Item) -> Result<(), ItemError> {
        self.contents.try_insert(item)
    }

    fn take_out(&mut self, id: &str) -> Result<Item, ItemError> {
        self.contents.take_out(id)
    }
    
    fn items(&self) -> &crate::item::ItemMap {
        self.contents.items()
    }
}
