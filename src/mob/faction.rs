//! Mob "factions"…
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Behavior {
    Friendly,
    Neutral,
    Hostile,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum MobFaction {
    Beast { behavior: Behavior },

    Vendor,
    Guard,
}

impl MobFaction {
    pub fn default_behavior(&self) -> Behavior {
        match self {
            Self::Beast { behavior } => *behavior,
            Self::Vendor => Behavior::Friendly,
            Self::Guard => Behavior::Neutral,
        }
    }
}
