use crate::item::{item::Item, ItemError};

pub trait StorageCapacity {
    fn space(&self) -> usize;
    fn num_items(&self) -> usize {0}
    fn capacity(&self) -> usize {0}
}

pub trait Storage {
    fn try_insert(&mut self, item: Item) -> Result<(), ItemError>;
    fn take_out(&mut self, id: &str) -> Result<Item, ItemError>;
}

pub trait Identity {
    fn is_container(&self) -> bool;
}
