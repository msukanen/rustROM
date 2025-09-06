use serde::{Deserialize, Serialize};

pub(crate) mod storage;
pub(crate) use storage::{Storage, StorageCapacity};
pub(crate) mod content;
pub(crate) use content::Content;
use crate::{item::{inventory::storage::Identity as StorageId, item::Item, ItemError}, traits::{owned::{OwnerError, UNSPECIFIED_OWNER}, Identity, Owned}};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ContainerType {
    Backpack,
    PlayerInventory,
    Room(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Container {
    Backpack(Content),
    PlayerInventory(Content),
    Room(Content),
}

// impl Default to appease [serde(default)] tags.
impl Default for Container {
    fn default() -> Self { Self::from(ContainerType::PlayerInventory) }
}

impl StorageCapacity for Container {
    fn capacity(&self) -> usize {
        match self {
            Self::Backpack(c)|
            Self::PlayerInventory(c)|
            Self::Room(c) => c.capacity(),
        }
    }

    fn num_items(&self) -> usize {
        match self {
            Self::Backpack(c)|
            Self::PlayerInventory(c)|
            Self::Room(c) => c.num_items(),
        }
    }

    fn space(&self) -> usize {
        match self {
            Self::Backpack(c)|
            Self::PlayerInventory(c)|
            Self::Room(c) => c.space(),
        }
    }
}

impl Storage for Container {
    fn try_insert(&mut self, item: Item) -> Result<(), ItemError> {
        match self {
            Self::Backpack(c)|
            Self::PlayerInventory(c)|
            Self::Room(c)
                => c.try_insert(item),
        }
    }

    fn take_out(&mut self, id: &str) -> Result<Item, ItemError> {
        match self {
            Self::Backpack(c)|
            Self::PlayerInventory(c)|
            Self::Room(c)
                => c.take_out(id),
        }
    }

    fn items(&self) -> &super::ItemMap {
        match self {
            Self::Backpack(c)|
            Self::PlayerInventory(c)|
            Self::Room(c)
                => c.items()
        }
    }

    fn items_mut(&mut self) -> &mut super::ItemMap {
        match self {
            Self::Backpack(c)|
            Self::PlayerInventory(c)|
            Self::Room(c)
                => c.items_mut()
        }
    }

    fn contains(&self, id: &str) -> bool {
        match self {
            Self::Backpack(c)|
            Self::PlayerInventory(c)|
            Self::Room(c)
                => c.contains(id)
        }
    }

    fn contains_r(&self, id: &str) -> Result<String, String> {
        match self {
            Self::Backpack(c)|
            Self::PlayerInventory(c)|
            Self::Room(c)
                => c.contains_r(id)
        }
    }
}

impl Identity for Container {
    fn id<'a>(&'a self) -> &'a str {
        match self {
            Self::Backpack(c)|
            Self::PlayerInventory(c)|
            Self::Room(c)
                => c.id()
        }
    }
}

impl From<ContainerType> for Container {
    fn from(value: ContainerType) -> Self {
        match &value {
            ContainerType::PlayerInventory => Container::PlayerInventory(Content::from(ContainerType::PlayerInventory)),
            ContainerType::Room(_) => {
                Container::Room(Content::from(value))
            },
            _ => unimplemented!("more match arms needed"),
        }
    }
}

impl StorageId for Container {
    fn is_container(&self) -> bool { true }
}

impl Owned for Container {
    fn owner(&self) -> &str {
        match self {
            Self::Backpack(c)|
            Self::PlayerInventory(c) => c.owner(),
            Self::Room(_) => UNSPECIFIED_OWNER,
        }
    }

    fn set_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> {
        match self {
            Self::Backpack(c) => c.set_owner(owner_id),
            Self::PlayerInventory(c) => {
                // setting ownership of PlayerInventory is a one-time process - generally happening during load/new of Player.
                if !c.is_owned() {
                    c.set_owner(owner_id);
                    c.set_original_owner(owner_id);
                    Ok(())
                } else {
                    log::error!("A logic error somewhere - an attempt to change ownership of PlayerInventory… Which is naturally rejected.");
                    Err(OwnerError::ImmutableOwnership)
                }
            },
            Self::Room(_) => Err(OwnerError::NotOwnable),
        }
    }

    fn original_owner(&self) -> &str {
        match self {
            Self::Backpack(c) => c.original_owner(),
            Self::PlayerInventory(c) => c.owner(),
            Self::Room(_) => UNSPECIFIED_OWNER,
        }
    }

    fn set_original_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> {
        match self {
            Self::Backpack(c)|
            Self::PlayerInventory(c) => c.set_original_owner(owner_id),
            Self::Room(_) => Err(OwnerError::ImmutableOwnership),
        }
    }
}

#[cfg(feature = "localtest")]
impl Container {
    pub(crate) fn set_id(&mut self, id: &str) {
        unimplemented!("No set_id() defined.");
    }
}

#[cfg(test)]
mod inventory_tests {
    use crate::item::item::ItemType;

    use super::*;

    #[test]
    fn basic_inventory() {
        let inv = Container::default();
        assert!(inv.is_container());
    }

    #[test]
    fn put_item_in_inventory() {
        let mut inv = Container::default();
        let item = Item::new(ItemType::Weapon);
        let res = inv.try_insert(item);
        match res {
            Err(ItemError::NoSpace(_)) => panic!("out of space?"),
            Err(ItemError::NotContainer(_)) => panic!("oopsie - inv is NOT a container?!"),
            Err(ItemError::TooLarge(_)) => panic!("yeah, that doesn't go there..."),
            _ => {}
        }
        assert_eq!(1, inv.num_items());
    }

    #[test]
    #[should_panic]
    fn spam_put_item_in_inventory_and_panic() {
        const SPAM_COUNT: usize = 10_000;
        let mut inv = Container::default();
        let item = Item::new(ItemType::Weapon);
        for x in 1..=SPAM_COUNT {
            let item = item.clone().re_id().clone();
            if let Err(_) = inv.try_insert(item) {
                panic!("Could not fit item #{}; capacity of {} exceeded", x, inv.capacity());
            }
        }
    }

    #[test]
    fn take_item_from_inventory() {
        let mut inv = Container::default();
        let item = Item::new(ItemType::Weapon);
        let id = item.id().to_string();
        let _ = inv.try_insert(item);
        let res = inv.take_out(&id);
        if let Ok(_) = res {
            let res = inv.take_out(&id);
            if let Err(ItemError::NotFound) = res {}
            else {
                panic!("Item is stuck in inventory?");
            }
        } else {
            panic!("{:?}", res)
        }
    }
}
