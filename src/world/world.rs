//! The World
//! 
//! Note about areas and rooms that the world HAS TO HAVE:
//! - an area called `root` (`data/areas/root.area`).
//! - a room called `root` (`data/areas/root.room`).
//! 
//! The dual `root:root` is used as an entrance for new players,
//! guests, and as a fallback after major world changes which
//! cause e.g. saved locations in player saves to be invalid.
//! 
//! If one or the other file is missing… Bad Things™ will happen!
use std::{collections::HashMap, fs::read_to_string, net::SocketAddr, path::PathBuf, str::FromStr, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{cmd::CommandCtx, player::Player, string::prompt::PromptType, traits::tickable::Tickable, util::{contact::{AdminInfo, Contact}, help::Help}, world::{area::{world_area_serialization, Area}, room::Room}, DATA_PATH};

#[derive(Debug, Deserialize, Serialize)]
pub struct MotD {
    text: String,
}

/// World entrance.
/// 
/// Used for locating e.g. players, mobs, rooms themselves, etc.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorldEntrance {
    pub area: String,
    pub room: String,
}

impl WorldEntrance {
    /// Generate default entrance entry.
    pub fn default() -> Self {
        Self {
            area: "root".into(),
            room: "root".into()
        }
    }

    /// A blank entrance.
    pub fn new() -> Self {
        Self { area: "".into(), room: "".into() }
    }
}

/// The World itself…
#[derive(Debug, Deserialize, Serialize)]
pub struct World {
    #[serde(default)] uptime: u64,
    #[serde(skip)] filename: String,
    title: String,
    description: String,
    owner: Contact,
    admins: Option<Vec<AdminInfo>>,
    pub motd: Option<Vec<MotD>>,
    pub greeting: Option<String>,
    pub welcome_back: Option<String>,
    pub welcome_new: Option<String>,
    #[serde(with = "world_area_serialization")]
    pub areas: HashMap<String, Arc<RwLock<Area>>>,
    pub root: WorldEntrance,
    pub prompts: HashMap<PromptType, String>,
    #[serde(skip, default)] pub players: HashMap<SocketAddr, Arc<RwLock<Player>>>,
    #[serde(skip, default)] pub players_to_logout: Vec<Arc<RwLock<Player>>>,
    #[serde(skip, default)] pub rooms: HashMap<String, Arc<RwLock<Room>>>,
    #[serde(skip, default)] pub help: HashMap<String, Arc<RwLock<Help>>>,
    #[serde(skip, default)] pub help_aliased: HashMap<String, String>,
}

/// Thread-shared world type.
pub type SharedWorld = Arc<RwLock<World>>;

#[derive(Debug)]
pub enum WorldError {
    Io(std::io::Error),
    Format(serde_json::Error),
}

impl From<std::io::Error> for WorldError {
    fn from(value: std::io::Error) -> Self { Self::Io(value)}
}

impl From<serde_json::Error> for WorldError {
    fn from(value: serde_json::Error) -> Self { Self::Format(value)}
}

impl World {
    /// A brand new world (loaded from a file, of course).
    /// 
    /// # Arguments
    /// - `name`— stem-name of the world. Designates the storage medium (without filename extension, etc.).
    pub async fn new(name: &str) -> Result<Self, WorldError> {
        let filename = format!("{}/{}.world", *DATA_PATH, name);
        log::info!("Loading '{}'", filename);
        let path = PathBuf::from_str(filename.as_str()).unwrap();
        let content = if !path.exists() {
            let world = World::bootstrap(name).await;
            if let Err(_) = world {
                panic!("Oh dear! Could not generate world skeleton! Abort!");
            }
            world.unwrap()
        } else {
            read_to_string(path)?
        };
        let mut world: World = serde_json::from_str(&content)?;
        world.filename = filename;
        Ok(world)
    }

    /// A blank world for blank reasons...
    #[cfg(test)]
    pub(crate) fn blank() -> Self { Self {
        uptime: 0,
        filename: "/dev/null".into(),
        title: "".into(),
        description: "".to_string(),
        owner: Contact::new(),
        admins: None,
        motd: None,
        greeting: None,
        welcome_back: None,
        welcome_new: None,
        areas: HashMap::new(),
        root: WorldEntrance::new(),
        prompts: HashMap::new(),
        players: HashMap::new(),
        players_to_logout: vec![],
        rooms: HashMap::new(),
        help: HashMap::new(),
        help_aliased: HashMap::new(),
    }}

    /// Bootstrap MUD from grounds up.
    pub async fn bootstrap(name: &str) -> Result<String, std::io::Error> {
        log::warn!("Bootstrapping - no previous world setup detected …");
        tokio::fs::create_dir_all((*DATA_PATH).as_str()).await?;
        // Bootstrap the "subsystems"…
        Player::bootstrap().await?;
        Room::bootstrap().await?;
        Area::bootstrap().await?;
        log::warn!("Bootstrap - generating world skeleton '{}/{}.world'", *DATA_PATH, name);
        let world = serde_json::json!({
            "title": "RustROM World",
            "description": "A World To Be",
            "owner": {
                "name": "The Owner",
                "email": "owner.of@the.world"
            },
            "admins": [{}],
            "motd": [],
            "greeting": "Welcome to your new RustROM!",
            "welcome_back": "Welcome back!",
            "welcome_new": "Welcome, new adventurer!",
            "areas": ["root"],
            "root": {
                "area": "root",
                "room": "root"
            },
            "prompts": {}
        });
        let world = serde_json::to_string_pretty(&world)?;
        tokio::fs::write(format!("{}/{}.world", *DATA_PATH, name), &world).await?;
        log::info!("Bootstrap({}.world) OK.", name);
        Ok(world)
    }

    #[cfg(test)]
    pub async fn do_busy_stuff(&self) {
        use std::time::Duration;

        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    /// Validate the integrity of the loaded world data.
    #[must_use = "This result must be checked to ensure world integrity."]
    pub async fn validate(self) -> Result<Self, String> {
        let mut r: Option<Result<Self, String>> = None;
        // See that the root area and room actually exist.
        if let Some(area) = self.areas.get(&self.root.area) {
            if area.read().await.rooms.get(&self.root.room).is_none() {
                r = Some(Err(format!(
                    "Validation error: root room '{}' defined in '{}' does not exist.",
                    self.root.room, self.filename
                )));
            }
        } else {
            r = Some(Err(format!(
                "Validation error: root area '{}' defined in '{}' does not exist.",
                self.root.area, self.filename
            )))
        }
        r.or(Some(Ok(self))).unwrap()
    }

    /// Transfer to a valid room if 'room' is None.
    /// 
    /// # Returns
    /// `true` if transfer actually had to be done.
    pub async fn transfer_to_safety(&self, ctx: &mut CommandCtx<'_>, room: &Option<Arc<Room>>) -> bool {
        if room.is_some() {false}
        else {
            ctx.player.write().await.location = "root".to_string();
            true
        }
    }
}

#[async_trait]
impl Tickable for World {
    async fn tick(&mut self, uptime: u64) {
        self.uptime = uptime;
        for area in self.areas.values() {
            area.write().await
                .tick(self.uptime).await;
        }
    }
}

#[cfg(test)]
mod world_tests {
    /// Let's see how the threads react to the core world being super busy with global locks.
    #[tokio::test]
    #[cfg(feature = "ittest")]
    async fn busy_world() {
        let _ = env_logger::try_init();
        let _ = crate::DATA.set("./data".into());
        let world = std::sync::Arc::new(
            tokio::sync::RwLock::new(
                crate::world::World::new("rustrom").await.expect("ERROR: world dead or in fire?!")
            )
        );

        tokio::spawn(crate::game_loop::game_loop(world.clone()));
        {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            log::debug!("Enter guard...");
            {
                let w = world.write().await;
                w.do_busy_stuff().await;
                log::debug!("Exit guard...");
            }
            log::debug!("Waited busy stuff...");
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
        log::debug!("Lazing about.");
    }
}
