use std::sync::Arc;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{DATA_PATH, traits::tickable::Tickable, world::room::{Room, rooms_serialization}};

pub(crate) static AREA_PATH: Lazy<Arc<String>> = Lazy::new(|| Arc::new(format!("{}/areas", *DATA_PATH)));

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Area {
    name: String,
    description: String,
    #[serde(with = "rooms_serialization", skip_serializing_if = "Option::is_none")]
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
