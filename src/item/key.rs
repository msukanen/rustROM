//! Keys!

use serde::{Deserialize, Serialize};

use crate::traits::{Description, Identity, Owned, owned::{Owner, OwnerError}};

// TODO: naming creativity!.
fn title_default() -> String { "a key".into() }

/// It's a me, a key!
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Key {
    /// ID of the key. Unlike with other items, keys generally are *not* tagged with UUID.
    id: String,
    #[serde(default)] owner: Owner,
    #[serde(default = "title_default")] title: String,
    #[serde(default = "title_default")] description: String,// = title_default…
    #[serde(default)] pub one_time: bool,
}

impl Identity for Key {
    fn id<'a>(&'a self) -> &'a str { &self.id }
    fn title<'a>(&'a self) -> &'a str { &self.title }
}

impl Key {
    #[cfg(test)]
    pub fn new(id: &str, one_time: bool) -> Self {
        Self {
            id: id.into(),
            one_time,
            owner: Owner::default(),
            title: title_default(),
            description: title_default(),
        }
    }
}

impl Owned for Key {
    fn owner(&self) -> &str { self.owner.owner() }
    fn original_owner(&self) -> &str { self.owner.original_owner() }
    fn set_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> { self.owner.set_owner(owner_id) }
    fn set_original_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> { self.owner.set_original_owner(owner_id) }
}

impl Description for Key {
    fn description<'a>(&'a self) -> &'a str { &self.description }
}
