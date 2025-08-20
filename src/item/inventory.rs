use serde::{Deserialize, Serialize};

use crate::{item::{ItemError, Item}, traits::{Description, Identity}};

/// Container core.
#[derive(Debug, Deserialize, Serialize)]
pub struct Container {
    id: String,
    title: String,
    description: String,
    
    capacity: u16,

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

impl Container {
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

    pub fn items(&self) -> usize {
        let mut count = self.items.len();
        for c in &self.subcontainers {
            count += c.items();
        }
        count + self.subcontainers.len()
    }

    pub fn space(&self) -> usize {
        self.capacity as usize - self.items()
    }
}
