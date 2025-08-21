use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{item::{Item, ItemError}, traits::{Description, Identity, Owned}};

pub(crate) trait StorageCapacity {
    fn space(&self) -> usize;
    fn items(&self) -> usize {0}
    fn capacity(&self) -> usize {0}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BaseContainerType {
    Pouch,
    Bag,
    Backpack,
    Trunk,
    Closet,
    StorageContainer,
    Warehouse,
    //
    PlayerInventory,
}

impl BaseContainerType {
    /// Generates a random ID.
    pub fn uuid(&self) -> String {
        format!("{}-{}", self.title(), Uuid::new_v4())
    }

    pub fn can_be_carried(&self) -> bool {
        self.capacity() <= Self::PlayerInventory.capacity()
    }
}

impl StorageCapacity for BaseContainerType {
    /// Base capacity of base container types.
    /// 
    // TODO: the values are entirely arbitrary at this point in time, and are bound to change Whenever™.
    fn capacity(&self) -> usize {
        match self {
            Self::Pouch => 8,
            Self::Bag => 24,
            Self::Backpack => 32,
            Self::Trunk => 64,
            Self::Closet => 128,
            Self::StorageContainer => 512,
            Self::Warehouse => 4096,
            //
            Self::PlayerInventory => (Self::Trunk.capacity() + Self::Backpack.capacity()) / 2,
        }
    }

    /// This function should not be called on a `BaseContainerType`.
    ///
    /// `BaseContainerType` is a template and does not hold items itself.
    /// This implementation will log an error and return 0.
    fn items(&self) -> usize {
        log::error!(".items() should not be called on [BaseContainerType]."); 0
    }

    /// This function should not be called on a `BaseContainerType`.
    ///
    /// `BaseContainerType` is a template and does not hold items itself.
    /// This implementation will log an error and return 0.
    fn space(&self) -> usize {
        log::error!(".space() should not be called on [BaseContainerType]."); 0
    }
}

/// Container core.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Container {
    id: String,
    title: String,
    description: String,
    base: BaseContainerType,

    owner: String,
    
    capacity: usize,

    items: Vec<Item>,
    subcontainers: Vec<Container>,
}

impl Identity for Container {
    fn id<'a>(&'a self) -> &'a str { &self.id }
}

impl Description for Container {
    fn description<'a>(&'a self) -> &'a str {
        &self.description
    }

    fn title<'a>(&'a self) -> &'a str {
        &self.title
    }
}

impl From<BaseContainerType> for Container {
    fn from(value: BaseContainerType) -> Self {
        Self::new("", value)
    }
}

impl Container {
    pub fn new(owner: &str, base: BaseContainerType) -> Self {
        Self {
            owner: owner.to_string(),
            id: base.uuid(),
            title: base.title().to_string(),
            description: base.description().to_string(),
            capacity: base.capacity(),
            base,
            items: vec![],
            subcontainers: vec![],
        }
    }

    /// Try insert an `item` into the container. In case of failure, `item` is
    /// returned along an [ItemError] and must be extracted from it or deemed
    /// lost forever.
    /// 
    /// # Arguments
    /// - `item`— some [Item].
    #[must_use = "If fails - item will be lost if not retrieved from Err."]
    pub fn try_insert(&mut self, item: Item) -> Result<(), ItemError> {
        if self.items.len() >= self.capacity() {
            return Err(ItemError::NoItemSpace(item));
        }
        self.items.push(item);
        Ok(())
    }

    /// Try insert a `container` into the container. In case of failure, `container` is
    /// returned along an [ItemError] and must be extracted from it or deemed
    /// lost forever.
    /// 
    /// Note that you cannot insert equal capacity or larger container into a smaller one…
    /// 
    /// # Arguments
    /// - `container`— some [Container].
    #[must_use = "If fails - container will be lost if not retrieved from Err."]
    pub fn try_insert_container(&mut self, container: Container) -> Result<(), ItemError> {
        if self.capacity <= container.capacity {
            return Err(ItemError::TooLarge(container));
        }
        if self.space() - 1 < container.items() {
            return Err(ItemError::NoContainerSpace(container));
        }
        self.subcontainers.push(container);
        Ok(())
    }
}

impl Owned for Container {
    fn owner(&self) -> &str { &self.owner }
}

impl StorageCapacity for Container {
    fn capacity(&self) -> usize { self.capacity }
    fn space(&self) -> usize { self.capacity - self.items() }
    /// Gets number of items in the container (+ what is contained in containers within).
    fn items(&self) -> usize {
        let mut count = self.items.len();
        for c in &self.subcontainers {
            count += c.items();
        }
        count + self.subcontainers.len()
    }
}

impl Description for BaseContainerType {
    fn description<'a>(&'a self) -> &'a str {
        match self {
            Self::Backpack => "backpack",
            Self::Bag => "bag",
            Self::Closet => "closet",
            Self::Pouch => "pouch",
            Self::StorageContainer => "large storage",
            Self::Trunk => "trunk",
            Self::Warehouse => "warehouse",
            //
            Self::PlayerInventory => "inventory",
        }
    }

    fn title<'a>(&'a self) -> &'a str {
        match self {
            Self::Backpack => "backpack",
            Self::Bag => "bag",
            Self::Closet => "closet",
            Self::Pouch => "pouch",
            Self::StorageContainer => "large storage",
            Self::Trunk => "trunk",
            Self::Warehouse => "warehouse",
            //
            Self::PlayerInventory => "inventory",
        }
    }
}

#[cfg(test)]
mod container_tests {
    use crate::{item::{inventory::{BaseContainerType, StorageCapacity}, Container, Item, ItemError}, traits::{Description, Identity}};

    #[test]
    fn inventory_created() {
        let _ = env_logger::try_init();
        let i = Container::from(BaseContainerType::PlayerInventory);
        assert_eq!("inventory", i.title());
    }

    #[test]
    fn inventory_insert_item() {
        let _ = env_logger::try_init();
        let item = Item::blank();
        let mut i = Container::from(BaseContainerType::PlayerInventory);
        let res = i.try_insert(item);
        match res {
            Err(e) => match e {
                ItemError::NoContainerSpace(_) => {
                    panic!("A bug in the system - [Item] insert should not result with Err(..container..)!");
                },
                ItemError::NoItemSpace(_) => {
                    panic!("Should not happen - out of inventory space?");
                },
                ItemError::TooLarge(_) => {
                    panic!("a blank() item should fit in {:?}", BaseContainerType::PlayerInventory);
                }
            }, _=> {}
        }
    }

    #[test]
    #[should_panic]
    fn inventory_insert_item_spam() {
        let _ = env_logger::try_init();
        let item = Item::blank();
        let mut items = vec![];
        const ENSURED_OVERFLOW: usize = 10_000;
        for _ in 0..ENSURED_OVERFLOW { items.push(item.clone()); }
        let mut inv = Container::from(BaseContainerType::PlayerInventory);
        for (xth, item) in items.into_iter().enumerate() {
            let res = inv.try_insert(item);
            if let Err(ItemError::NoItemSpace(_)) = res {
                log::debug!("Ran out of space at {} items (from {ENSURED_OVERFLOW}). Capacity of {}.", xth, inv.capacity());
                panic!("Whee!")
            }
        }
        panic!("Bag of Holding?! Should NOT happen!");
    }

    #[test]
    fn basecontainertype_log_error() {
        let _ = env_logger::try_init();
        let item = BaseContainerType::Backpack;
        let x = item.space();
        if x != 0 {
            panic!("BaseContainerType should return 0 here!");
        }
    }
}
