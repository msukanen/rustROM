use serde::{Deserialize, Serialize};

mod melee;
use melee::MeleeInfo;
mod ranged;
use ranged::RangedInfo;

use crate::traits::Identity;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum WeaponType {
    Melee,
    Ranged,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Weapon {
    Melee(MeleeInfo),
    Ranged(RangedInfo,)
}

impl Identity for Weapon {
    fn id<'a>(&'a self) -> &'a str {
        match self {
            Self::Melee(m) => m.id(),
            Self::Ranged(r) => r.id(),
        }
    }
}
