//! "Make room!" - the [Room] live here.
use std::{collections::{HashMap, HashSet, VecDeque}, fmt::Display, fs, path::PathBuf, sync::{Arc, Weak}};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tokio::sync::RwLock;

use crate::{DATA_PATH, item::{Item, ItemError, inventory::{Container, ContainerType, Storage, StorageCapacity}}, player::Player, traits::{Description, Identity, save::{DoesSave, SaveError}}, util::{Editor, direction::Direction}, world::{SharedWorld, area::Area}};

pub(crate) static ROOM_PATH: Lazy<Arc<String>> = Lazy::new(|| Arc::new(format!("{}/rooms", *DATA_PATH)));
/// Max number of items in a [Room], whether on ground or otherwise.
pub(crate) static MAX_ITEMS_IN_ROOM: usize = 1_000;

pub enum RoomError {
    NoRoom,
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

impl PartialEq for Exit {
    fn eq(&self, other: &Self) -> bool {
        self.destination == other.destination
    }
}

impl Display for Exit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.destination)
    }
}

impl From<&str> for Exit {
    fn from(destination: &str) -> Self {
        Self {
            destination: destination.into(),
            state: ExitState::default(),
        }
    }
}

// Just a "lazy convenience" From<>…
impl From<&&str> for Exit {
    fn from(value: &&str) -> Self {
        (*value).into()
    }
}

impl Default for ExitState {
    fn default() -> Self {
        ExitState::Open
    }
}

#[inline]
fn room_parent_id_default() -> String {"root".into()}

// NOTE: keep this and deserializer's RoomFile struct in sync!
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Room {
    pub id: String,
    pub title: String,
    pub description: String,
    pub exits: HashMap<Direction, Exit>,
    
    /// Parent [Area] ID.
    #[serde(default = "room_parent_id_default")]
    pub parent_id: String,
    
    /// Weak lock to parent [Area]; set elsewhere.
    #[serde(skip)]
    pub parent: Weak<RwLock<Area>>,
    
    /// Weak lock to [Player] entities currently present in the [Room].
    #[serde(skip, default)]
    pub players: HashMap<String, Weak<RwLock<Player>>>,
    
    /// [Room] [contents][Container]… a.k.a. whatever lies around.
    pub contents: Container,
}

impl <'de> Deserialize<'de> for Room {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        #[derive(Deserialize)]
        struct RoomFile {
            id: String,
            title: String,
            description: String,
            exits: HashMap<Direction, Exit>,
            #[serde(default = "room_parent_id_default")]
            parent_id: String,
            contents: Option<Container>,
        }

        let rf: RoomFile = Deserialize::deserialize(deserializer)?;
        Ok(Self {
            title: rf.title,
            description: rf.description,
            exits: rf.exits,
            parent_id: rf.parent_id,
            parent: Weak::new(),
            contents: rf.contents.unwrap_or_else(|| Room::default_container(&rf.id)),
            players: HashMap::new(),
            id: rf.id,
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
            title: "".into(),
            description: "".into(),
            exits: HashMap::new(),
            parent_id: "root".into(),
            parent: Weak::new(),
            players: HashMap::new(),
            contents: Room::default_container(&id),
            id,
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
        true
    }

    /// Remove [Player] from the [Room].
    /// 
    /// Note that it is *not* considered an error to try remove [Player] which is
    /// not in this particular room. We'll just silently ignore the call.
    /// 
    /// # Arguments
    /// - `player`— Some [Player].
    pub async fn remove_player(&mut self, player: &Arc<RwLock<Player>>) {
        let lock = player.read().await;
        let id = lock.id();
        if let Some(_) = self.players.remove(id) {
            log::debug!("Player '{}' removed from room '{}'", id, self.id());
        }
    }

    /// Generate a Room-[Container] for `id`.
    fn default_container(id: &str) -> Container {
        Container::from(ContainerType::Room(id.into()))
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

    fn contains(&self, id: &str) -> bool {
        self.contents.contains(id)
    }

    fn contains_r(&self, id: &str) -> Result<String, String> {
        self.contents.contains_r(id)
    }

    fn items_mut(&mut self) -> &mut crate::item::ItemMap {
        self.contents.items_mut()
    }

    fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }
}

#[async_trait]
impl DoesSave for Room {
    async fn save(&mut self) -> Result<(), SaveError> {
        let path = PathBuf::from(&format!("{}/{}.room", *ROOM_PATH, self.id()));
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

impl Room {
    pub(crate) fn shallow_copy(&mut self, other: &Self) {
        self.id = other.id.clone();
        self.description = other.description.clone();
        self.title = other.title.clone();
        self.exits = other.exits.clone();
    }
}

#[cfg(test)]
mod room_tests {
    use crate::world::room::Room;

    /// See that [Room] and [RoomFile] stay in sync…
    #[test]
    fn test_room_serialization_sync() {
        let room_json = r#"{
            "id": "nexus",
            "title": "The Nexus",
            "description": "Center of the world.",
            "exits": {},
            "parent_id": "root"
        }"#;
        
        let room: Room = serde_json::from_str(room_json).unwrap();
        assert_eq!(room.id, "nexus");
    }    
}
