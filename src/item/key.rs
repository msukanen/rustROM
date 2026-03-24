//! Keys!

use serde::{Deserialize, Serialize};

use crate::traits::{Identity, Owned, owned::{Owner, OwnerError}};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Key {
    id: String,
    owner: Owner,
    pub one_time: bool,
}

impl Identity for Key {
    fn id<'a>(&'a self) -> &'a str {
        &self.id
    }
}

impl Key {
    #[cfg(test)]
    pub fn new(id: &str, one_time: bool) -> Self {
        Self { id: id.into(), one_time, owner: Owner::default() }
    }
}

impl Owned for Key {
    fn owner(&self) -> &str { self.owner.owner() }
    fn original_owner(&self) -> &str { self.owner.original_owner() }
    fn set_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> { self.owner.set_owner(owner_id) }
    fn set_original_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> { self.owner.set_original_owner(owner_id) }
}
