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
    fn get(&self, id: &str) -> Option<&Item>;
    /// See if something *resembling* `id` is contained within.
    /// 
    /// Note: **much** slower than [`Storage::contains`].
    fn contains_r(&self, id: &str) -> Result<String, String>;
    /// See if some blueprint `id` is contained within.
    fn contains_bp(&self, id: &str) -> bool;
    /// Get reference to the whole [ItemMap] within.
    fn items(&self) -> &ItemMap;
    /// Get mut reference to the whole [ItemMap] within.
    fn items_mut(&mut self) -> &mut ItemMap;
    fn is_empty(&self) -> bool;
}

/// Introducing identity crisis for all sorts of storage.
pub trait StorageIdentity {
    /// Check if this thing actually is a container at all…
    fn is_container(&self) -> bool;
}
