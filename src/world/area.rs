//! Area stuff.
use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::{Arc, Weak}};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::RwLock};

use crate::{DATA_PATH, traits::{Description, Identity, save::{DoesSave, SaveError}, tickable::Tickable}, world::{World, room::Room}};

pub(crate) static AREA_PATH: Lazy<Arc<String>> = Lazy::new(|| Arc::new(format!("{}/areas", *DATA_PATH)));
const DEFAULT_TICK_MODULO: u8 = 10;// normally an Area acts once every 10th tick.

const fn default_area_tick_modulo() -> u8 {DEFAULT_TICK_MODULO}// to appease 'serde(default = ...)'
#[derive(Debug, Deserialize, Serialize)]
pub struct Area {
    pub id: String,
    title: String,
    description: String,
    
    #[serde(skip)] pub parent: Weak<RwLock<World>>,
    #[serde(skip)] pub rooms: HashMap<String, Weak<RwLock<Room>>>,
    
    #[serde(default = "default_area_tick_modulo")]
    tick_modulo: u8,
}

#[async_trait]
impl Tickable for Area {
    async fn tick(&mut self, uptime: u64) {
        // Time to tick?
        if (uptime % self.tick_modulo as u64) != 0 {return ;}
    }
}

impl Area {
    /// Bootstrap - staging area.
    pub async fn bootstrap() -> Result<(), std::io::Error> {
        let stem = "root";
        log::warn!("Bootstrap - generating starter area '{}/{}.area'…", *AREA_PATH, stem);
        tokio::fs::create_dir_all((*AREA_PATH).as_str()).await?;
        let area = serde_json::json!({
            "name": "root",
            "title": "The Genesis Area",
            "description": "Where it all begins …",
            "rooms": ["root", "not-so-root"]
        });
        tokio::fs::write(format!("{}/{}.area", *AREA_PATH, stem), serde_json::to_string_pretty(&area)?).await?;
        log::info!("Bootstrap({}.area) OK.", stem);
        Ok(())
    }

    #[cfg(test)]
    /// A blank slate.
    pub(crate) fn blank() -> Self { Self {
        id: "".into(),
        title: "".into(),
        description: "".into(),
        rooms: HashMap::new(),
        parent: Weak::new(),
        tick_modulo: 10,
    }}
}

impl Description for Area {
    fn description(&self) -> &str { &self.description }
    fn title(&self) -> &str { &self.title }
}

impl Identity for Area {
    fn id(&self) -> &str { &self.id }
}

#[async_trait]
impl DoesSave for Area {
    /// Save the [Area]!
    async fn save(&mut self) -> Result<(), SaveError> {
        let path = PathBuf::from_str(&format!("{}/{}.area", *AREA_PATH, self.id())).unwrap();
        fs::write(path, serde_json::to_string_pretty(self)?).await?;
        Ok(())
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
        debug!("Area name: \"{}\"", area.id);
        debug!("     desc: \"{}\"", area.description);
        
    }
}
