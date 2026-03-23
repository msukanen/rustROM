use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{item::{Item, ItemError, ItemMap, inventory::{ContainerType, Storage, StorageCapacity, storage::StorageIdentity}}, player::pc::MAX_ITEMS_PLAYER_INVENTORY, traits::{Identity, Owned, owned::{Owner, OwnerError}}, world::room::MAX_ITEMS_IN_ROOM};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Content {
    id: String,
    #[serde(default)]
    owner: Owner,
    max_capacity: usize,
    contents: ItemMap,
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
                id: format!("content-pc-inv-{}", Uuid::new_v4()),
                owner: Owner::default(),
                max_capacity: MAX_ITEMS_PLAYER_INVENTORY,
                contents: HashMap::new()
            },
            ContainerType::Room(id) => {
                log::debug!("Creating room inventory.");

                Self {
                    // NOTE: id "must" be set later by the [Room] itself.
                    id: format!("content-room-{}", id),
                    owner: Owner::default(),
                    max_capacity: MAX_ITEMS_IN_ROOM,
                    contents: HashMap::new()
                }
            },
            ContainerType::Backpack => Self {
                id: format!("content-backpack-{}", Uuid::new_v4()),
                owner: Owner::default(),
                max_capacity: 32,// TODO NOTE: arbitrary value.
                contents: HashMap::new()
            },
        }
    }
}

impl Storage for Content {
    fn try_insert(&mut self, item: Item) -> Result<(), ItemError> {
        if !self.is_container() {
            log::trace!("Amusing attempt, but '{}' is not a container and thus cannot hold '{}'", self.id(), item.id());
            return Err(ItemError::NotContainer(item));
        }

        let c = item.num_items() + 1;
        if self.space() < c {
            log::debug!("No space left in container.");
            return Err(ItemError::NoSpace(item));
        }

        log::debug!("'{}' put into '{}'.", item.id(), self.id());
        self.contents.insert(item.id().into(), item);
        Ok(())
    }

    fn take_out(&mut self, id: &str) -> Result<Item, ItemError> {
        let id = id.trim().to_lowercase();
        
        // Swift extract if `id` happened to be one of the keys…
        if let Some(item) = self.contents.remove(&id) {
            log::debug!("'{id}' removed from '{}'.", self.id());
            return Ok(item);
        }

        // Search by e.g. title… slowly and unsurely ;-)
        let mut found = None;
        for (c_id, item) in &mut self.contents {
            if c_id.contains(&id)
            || item.owner().contains(&id) {
                found = Some(item.id().to_string());
                break;
            }
        }

        if let Some(f_id) = found {
            log::debug!("'{f_id}' removed from '{}'.", self.id());
            return Ok(self.contents.remove(&f_id).unwrap());
        } else {
            log::trace!("Nothing resembling '{id}' found in '{}'…", self.id());
        }

        Err(ItemError::NotFound)
    }

    fn items(&self) -> &ItemMap {
        &self.contents
    }

    fn items_mut(&mut self) -> &mut ItemMap {
        &mut self.contents
    }

    fn contains(&self, id: &str) -> bool {
        // 'id' may or may not be preprocessed, so…
        self.contents.contains_key(id.trim().to_lowercase().as_str())
    }

    fn contains_r(&self, id: &str) -> Result<String, String> {
        // 'id' should be clean trimmed by callsite, but we can't quite trust that…
        let id = id.trim().to_lowercase();
        let trim_id = id.as_str();
        for k in self.contents.keys() {
            if k.contains(trim_id) {
                return Ok(k.clone());
            }
        }
        Err(format!("No key matching with '{id}' found."))
    }

    fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }
}

impl Owned for Content {
    fn owner(&self) -> &str { self.owner.owner() }
    fn original_owner(&self) -> &str { self.owner.original_owner() }
    fn set_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> { self.owner.set_owner(owner_id) }
    fn set_original_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> { self.owner.set_original_owner(owner_id) }
}

impl StorageIdentity for Content {
    fn is_container(&self) -> bool {
        // Obviously, things with zero `max_capacity` are not considered containers at all.
        self.max_capacity > 0
    }
}
