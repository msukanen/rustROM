use crate::item::{item::Item, ItemError};

pub trait StorageCapacity {
    fn space(&self) -> usize;
    fn num_items(&self) -> usize {0}
    fn capacity(&self) -> usize {0}
}

pub trait Storage {
    fn try_insert(&mut self, item: Item) -> Result<(), ItemError>;
}
