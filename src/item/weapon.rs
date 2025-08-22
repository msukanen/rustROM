use serde::{Deserialize, Serialize};

mod melee;
use melee::MeleeInfo;
mod ranged;
use ranged::RangedInfo;

use crate::traits::{Identity, Owned};

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

impl Weapon {
    pub fn new(weapon_type: WeaponType) -> Self {
        match weapon_type {
            WeaponType::Melee => Self::Melee(MeleeInfo::default()),
            _ => unimplemented!("more match arms needed"),
        }
    }

    #[cfg(test)]
    pub(crate) fn re_id(&mut self) -> &Self {
        match self {
            Self::Melee(m) => m.re_id(),
            _ => unimplemented!("no test for other but Melee atm"),
        };
        self
    }
}

impl Owned for Weapon {
    fn owner(&self) -> &str {
        match self {
            Self::Melee(m) => m.owner(),
            _ => unimplemented!("more match arms needed"),
        }
    }
}
