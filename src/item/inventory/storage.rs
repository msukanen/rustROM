use crate::item::{Item, ItemMap, ItemError};

pub trait StorageCapacity {
    fn space(&self) -> usize;
    fn num_items(&self) -> usize {0}
    fn capacity(&self) -> usize {0}
}

/// A common trait for any storage item.
pub trait Storage {
    /// Attempt to insert `item`.
    fn try_insert(&mut self, item: Item) -> Result<(), ItemError>;
    /// Attempt to take out `id`.
    fn take_out(&mut self, id: &str) -> Result<Item, ItemError>;
    /// See if some `id` is contained within.
    fn contains(&self, id: &str) -> bool;
    /// See if something *resembling* `id` is contained within.
    /// 
    /// Note: **much** slower than [`Storage::contains`].
    fn contains_r(&self, id: &str) -> Result<String, String>;
    /// Get reference to the whole [ItemMap] within.
    fn items(&self) -> &ItemMap;
    /// Get mut reference to the whole [ItemMap] within.
    fn items_mut(&mut self) -> &mut ItemMap;
}

pub trait Identity {
    fn is_container(&self) -> bool;
}
