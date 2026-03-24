pub mod inventory;
pub mod item;
pub use item::{Item, ItemError, ItemMap};

use crate::traits::Identity;
pub mod weapon;
pub mod key;

/// A trait for anything and everything with "blueprint" of sorts.
pub trait BlueprintID : Identity {
    /// Get the blueprint ID.
    //
    // Blueprint ID ≠ ID, except in case of truly unique items.
    //
    fn bp_id<'a>(&'a self) -> &'a str;
}
