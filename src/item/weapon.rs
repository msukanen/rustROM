use serde::{Deserialize, Serialize};

mod melee;
use melee::MeleeInfo;
mod ranged;
use ranged::RangedInfo;

use crate::traits::{owned::OwnerError, Identity, Owned};

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
            Self::Ranged(r) => r.owner(),
        }
    }

    fn original_owner(&self) -> &str {
        match self {
            Self::Melee(m) => m.original_owner(),
            Self::Ranged(r) => r.original_owner(),
        }
    }

    fn set_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> {
        match self {
            Self::Melee(m) => m.set_owner(owner_id),
            Self::Ranged(r) => r.set_owner(owner_id),
        }
    }

    fn set_original_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> {
        match self {
            Self::Melee(m) => m.set_original_owner(owner_id),
            Self::Ranged(r) => r.set_original_owner(owner_id),
        }
    }
}

impl From<WeaponType> for Weapon {
    fn from(value: WeaponType) -> Self {
        match value {
            WeaponType::Melee => Self::Melee(MeleeInfo::default()),
            WeaponType::Ranged => Self::Ranged(RangedInfo::default()),
        }
    }
}

#[cfg(feature = "localtest")]
impl Weapon {
    pub(crate) fn set_id(&mut self, id: &str) {
        match self {
            Weapon::Melee(m) => m.set_id(id),
            _ => unimplemented!("set_id() is defined only for Weapon::Melee.")
        }
    }
}