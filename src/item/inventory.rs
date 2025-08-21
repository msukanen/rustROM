use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{item::{Item, ItemError}, traits::{Description, Identity, Owned}};

pub(crate) type StorageCapacity = usize;

#[derive(Debug, Deserialize, Serialize)]
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
    /// Base capacity of base container types.
    /// 
    // TODO: the values are entirely arbitrary at this point in time, and are bound to change Whenever™.
    pub fn capacity(&self) -> usize {
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

    /// Generates a random ID.
    pub fn uuid(&self) -> String {
        format!("{}-{}", self.title(), Uuid::new_v4())
    }

    pub fn can_be_carried(&self) -> bool {
        self.capacity() <= Self::PlayerInventory.capacity()
    }
}

/// Container core.
#[derive(Debug, Deserialize, Serialize)]
pub struct Container {
    id: String,
    title: String,
    description: String,
    base: BaseContainerType,

    owner: String,
    
    capacity: StorageCapacity,

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

    /// Get max. number of items the container can hold.
    pub fn max_items(&self) -> usize { self.capacity.into() }

    /// Try insert an `item` into the container. In case of failure, `item` is
    /// returned along an [ItemError] and must be extracted from it or deemed
    /// lost forever.
    /// 
    /// # Arguments
    /// - `item`— some [Item].
    #[must_use = "If fails - item will be lost if not retrieved from Err."]
    pub fn try_insert(&mut self, item: Item) -> Result<(), ItemError> {
        if self.items.len() >= self.max_items() {
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

    /// Gets number of items in the container (+ what is contained in containers within).
    pub fn items(&self) -> StorageCapacity {
        let mut count = self.items.len();
        for c in &self.subcontainers {
            count += c.items();
        }
        (count + self.subcontainers.len()) as StorageCapacity
    }

    /// Gets capacity left.
    pub fn space(&self) -> StorageCapacity {
        self.capacity - self.items()
    }
}

impl Owned for Container {
    fn owner(&self) -> &str { &self.owner }
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
