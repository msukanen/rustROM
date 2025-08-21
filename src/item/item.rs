use serde::{Deserialize, Serialize};

use crate::{item::{inventory::{Container, Storage, StorageCapacity}, weapon::Weapon}, traits::{Description, Identity, Owned}};

#[derive(Debug)]
pub enum ItemError {
    NotContainer(Item),
    NoSpace(Item),
    TooLarge(Item),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ItemType {
    Container,
    Weapon,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Item {
    Container(Container),
    Weapon(Weapon),
}

impl Item {
    pub fn is_container(&self) -> bool {
        match self {
            Self::Container(_) => true,
            _ => false
        }
    }

    #[must_use = "Item will be lost if not extracted from Err in case of a failure."]
    pub fn try_insert(&mut self, item: Item) -> Result<(), ItemError> {
        match self {
            Self::Container(c) => c.try_insert(item),
            _ => Err(ItemError::NotContainer(item))
        }
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
        }
    }
}
