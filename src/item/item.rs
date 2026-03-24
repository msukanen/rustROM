use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{item::{BlueprintID, inventory::{Container, Storage, StorageCapacity, storage::StorageIdentity}, key::Key, weapon::{Weapon, WeaponType}}, traits::{Identity, Owned, owned::OwnerError}};

pub type ItemMap = HashMap<String, Item>;

#[derive(Debug, Deserialize, Serialize)]
pub enum ItemError {
    NotContainer(Item),
    NoSpace(Item),
    TooLarge(Item),
    NotFound,
}

impl std::error::Error for ItemError {}

impl Display for ItemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSpace(i) => write!(f, "No space for '{}'", i.id()),
            Self::NotContainer(_) => write!(f, "Cannot insert; recipient is not a container"),
            Self::TooLarge(i) => write!(f, "'{}' is too large to fit", i.id()),
            Self::NotFound => write!(f, "No such item found."),
        }
    }
}

impl Identity for ItemError {
    fn id<'a>(&'a self) -> &'a str {
        match self {
            Self::NoSpace(i)|
            Self::NotContainer(i)|
            Self::TooLarge(i)
                => i.id(),
            _ => unimplemented!("Attempt to retrieve ID of an unfound item…")
        }
    }

    fn title<'a>(&'a self) -> &'a str {
        match self {
            Self::NoSpace(i)|
            Self::NotContainer(i)|
            Self::TooLarge(i)
                => i.title(),
            _ => unimplemented!("Attempt to retrieve title of a non-existent item…")
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ItemType {
    Container,
    Weapon,
    Key,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Item {
    Container(Container),
    Weapon(Weapon),
    Key(Key),
}

impl Item {
    #[must_use = "Item will be lost if not extracted from Err in case of a failure."]
    pub fn try_insert(&mut self, item: Item) -> Result<(), ItemError> {
        match self {
            Self::Container(c) => c.try_insert(item),
            _ => Err(ItemError::NotContainer(item))
        }
    }

    /// Craft some new item…
    /// 
    /// # Args
    /// - `item_type` of the new item.
    pub fn new(item_type: ItemType) -> Self {
        match item_type {
            ItemType::Weapon => Self::Weapon(Weapon::new(WeaponType::Melee)),
            _ => unimplemented!("more match arms needed"),
        }
    }

    /// Re-ID the item.
    #[cfg(test)]
    pub(crate) fn re_id(mut self) -> Self {
        match &mut self {
            Self::Weapon(w) => w.re_id(),
            _ => unimplemented!("no re-id for other but Weapon atm"),
        };
        self
    }
}

impl StorageCapacity for Item {
    fn capacity(&self) -> usize {
        match self {
            Self::Container(c) => c.capacity(),
            _ => 0,
        }
    }

    fn num_items(&self) -> usize {
        match self {
            Self::Container(c) => c.num_items(),
            _ => 0,
        }
    }

    fn space(&self) -> usize {
        match self {
            Self::Container(c) => c.space(),
            _ => 0,
        }
    }
}

impl Identity for Item {
    fn id<'a>(&'a self) -> &'a str {
        match self {
            Self::Container(c) => c.id(),
            Self::Weapon(w) => w.id(),
            Self::Key(k) => k.id(),
        }
    }

    fn title<'a>(&'a self) -> &'a str {
        match self {
            Self::Container(c) => c.title(),
            Self::Weapon(w) => w.title(),
            Self::Key(k) => k.title(),
        }
    }
}

#[cfg(feature = "localtest")]
impl Item {
    pub(crate) fn set_id(&mut self, id: &str) {
        match self {
            Self::Weapon(w) => w.set_id(id),
            _ => unimplemented!("set_id() is defined only for Weapon.")
        }
    }
}

impl Owned for Item {
    fn owner(&self) -> &str {
        match self {
            Self::Container(c) => c.owner(),
            Self::Weapon(w) => w.owner(),
            Self::Key(k) => k.owner(),
        }
    }

    fn original_owner(&self) -> &str {
        match self {
            Self::Container(c) => c.original_owner(),
            Self::Weapon(w) => w.original_owner(),
            Self::Key(k) => k.original_owner(),
        }
    }

    fn set_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> {
        let owned = self.is_owned();
        match self {
            Self::Container(c) => {
                match c {
                    Container::Backpack(c) => c.set_owner(owner_id),
                    Container::PlayerInventory(c) => {
                        if owned {
                            log::warn!("Skipping attempt to change ownership of owned PlayerInventory.");
                            Err(OwnerError::ImmutableOwnership)
                        } else {
                            c.set_owner(owner_id)
                        }
                    },
                    Container::Room(_) => {
                        log::warn!("Cannot set owner for a Room. Immutable, unsettable.");
                        Err(OwnerError::ImmutableOwnership)
                    }
                }
            },
            Self::Weapon(w) => w.set_owner(owner_id),
            Self::Key(k) => k.set_owner(owner_id),
        }
    }

    fn set_original_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> {
        match self {
            Self::Container(c) => c.set_original_owner(owner_id),
            Self::Weapon(w) => w.set_original_owner(owner_id),
            Self::Key(k) => k.set_original_owner(owner_id),
        }
    }
}

impl From<WeaponType> for Item {
    fn from(value: WeaponType) -> Self {
        Self::Weapon(Weapon::from(value))
    }
}

impl From<ItemError> for Item {
    fn from(value: ItemError) -> Self {
        match value {
            ItemError::NoSpace(i)|
            ItemError::NotContainer(i)|
            ItemError::TooLarge(i) => i,
            ItemError::NotFound => panic!("Coder failure?")
        }
    }
}

/// Force item to player, if possible...
#[macro_export]
macro_rules! force_item_to_player {
    ($ctx:ident, $item:ident) => {{
        let e = $ctx.player.write().await.inventory.try_insert($item);
        if let Err(e) = e {
            log::error!("Could NOT force item to player '{}': {e}", $ctx.player.read().await.id());
        }
    }};
}

impl StorageIdentity for Item {
    fn is_container(&self) -> bool {
        matches!(self, Self::Container(_))
    }
}

impl BlueprintID for Item {
    fn bp_id<'a>(&'a self) -> &'a str {
        match self {
            Self::Container(c) => c.bp_id(),
            Self::Key(k) => k.bp_id(),
            Self::Weapon(w) => w.bp_id(),
        }
    }
}
