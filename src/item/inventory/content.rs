use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{item::{inventory::{ContainerType, Storage, StorageCapacity}, Item, ItemError}, player::pc::MAX_ITEMS_PLAYER_INVENTORY, traits::{Identity, Owned, owned::UNSPECIFIED_OWNER}};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Content {
    id: String,
    owner: String,
    max_capacity: usize,
    contents: HashMap<String, Item>,
}

impl StorageCapacity for Content {
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

impl Identity for Content { fn id<'a>(&'a self) -> &'a str { &self.id }}

impl From<ContainerType> for Content {
    fn from(value: ContainerType) -> Self {
        match value {
            ContainerType::PlayerInventory => Self {
                id: format!("inventory-{}", Uuid::new_v4()),
                owner: UNSPECIFIED_OWNER.into(),
                max_capacity: MAX_ITEMS_PLAYER_INVENTORY,
                contents: HashMap::new()
            },
            _ => unimplemented!("more match arms needed"),
        }
    }
}

impl Storage for Content {
    fn try_insert(&mut self, item: Item) -> Result<(), ItemError> {
        let c = item.num_items() + 1;
        if self.space() < c {
            return Err(ItemError::NoSpace(item));
        }

        self.contents.insert(item.id().into(), item);
        Ok(())
    }

    fn take_out(&mut self, id: &str) -> Result<Item, ItemError> {
        // Swift extract if `id` happened to be one of the keys…
        if let Some(item) = self.contents.remove(id) {
            return Ok(item);
        }

        // Search by e.g. title… slooowly…
        let mut found = None;
        for (c_id, item) in &mut self.contents {
            if c_id.contains(&id) 
            || item.owner().contains(&id) {
                found = Some(item.id().to_string());
                break;
            }
        }

        if let Some(f_id) = found {
            return Ok(self.contents.remove(&f_id).unwrap());
        }

        Err(ItemError::NotFound)
    }
}

impl Owned for Content {
    fn owner(&self) -> &str {
        &self.owner
    }
}