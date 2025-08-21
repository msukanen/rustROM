use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod storage;
pub(crate) use storage::{Storage, StorageCapacity};
use crate::{item::{item::Item, ItemError}, player::pc::MAX_ITEMS_PLAYER_INVENTORY, traits::{Description, Identity, Owned}};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ContainerType {
    Backpack,
    PlayerInventory,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Container {
    Backpack(Contents),
    PlayerInventory(Contents),
}

// impl Default to appease [serde(default)] tags.
impl Default for Container {
    fn default() -> Self { Self::from(ContainerType::PlayerInventory) }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Contents {
    id: String,
    max_capacity: usize,
    contents: HashMap<String, Item>,
}

impl StorageCapacity for Contents {
    fn capacity(&self) -> usize { self.max_capacity }
    fn num_items(&self) -> usize {
        let mut count = self.contents.len();
        for (_, item) in &self.contents {
            count += item.num_items();
        }
        count
    }
    fn space(&self) -> usize {
        self.capacity() - self.num_items()
    }
}

impl StorageCapacity for Container {
    fn capacity(&self) -> usize {
        match self {
            Self::Backpack(c) |
            Self::PlayerInventory(c) => c.capacity(),
        }
    }

    fn num_items(&self) -> usize {
        match self {
            Self::Backpack(c)|
            Self::PlayerInventory(c) => c.num_items(),
        }
    }

    fn space(&self) -> usize {
        match self {
            Self::Backpack(c)|
            Self::PlayerInventory(c) => c.space(),
        }
    }
}

impl Storage for Contents {
    fn try_insert(&mut self, item: Item) -> Result<(), ItemError> {
        let c = item.num_items() + 1;
        if self.space() < c {
            return Err(ItemError::NoSpace(item));
        }

        self.contents.insert(item.id().into(), item);
        Ok(())
    }
}

impl Storage for Container {
    fn try_insert(&mut self, item: Item) -> Result<(), ItemError> {
        match self {
            Self::Backpack(c) |
            Self::PlayerInventory(c) => c.try_insert(item),
        }
    }
}

impl Identity for Contents { fn id<'a>(&'a self) -> &'a str { &self.id }}
impl Identity for Container {
    fn id<'a>(&'a self) -> &'a str {
        match self {
            Self::Backpack(c) |
            Self::PlayerInventory(c) => c.id()
        }
    }
}

impl From<ContainerType> for Container {
    fn from(value: ContainerType) -> Self {
        match value {
            ContainerType::PlayerInventory => Container::PlayerInventory(Contents::from(ContainerType::PlayerInventory)),
            _ => unimplemented!("more match arms needed"),
        }
    }
}

impl From<ContainerType> for Contents {
    fn from(value: ContainerType) -> Self {
        match value {
            ContainerType::PlayerInventory => Self {
                id: format!("inventory-{}", Uuid::new_v4()),
                max_capacity: MAX_ITEMS_PLAYER_INVENTORY,
                contents: HashMap::new()
            },
            _ => unimplemented!("more match arms needed"),
        }
    }
}

#[cfg(test)]
mod inventory_tests {
    
}
