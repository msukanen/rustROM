use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::traits::{owned::{Owner, OwnerError}, Identity, Owned};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RangedInfo {
    id: String,
    #[serde(default)]
    owner: Owner,
}

impl Identity for RangedInfo { fn id<'a>(&'a self) -> &'a str { &self.id }}

// TODO: Default is very likely temporary - but we'll use it for testing for now.
impl Default for RangedInfo {
    fn default() -> Self {
        Self {
            id: format!("weapon-ranged-{}", Uuid::new_v4()),
            owner: Owner::default(),
        }
    }
}

impl Owned for RangedInfo {
    fn owner(&self) -> &str { self.owner.owner() }
    fn original_owner(&self) -> &str { self.owner.original_owner() }
    fn set_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> { self.owner.set_owner(owner_id) }
    fn set_original_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> { self.owner.set_original_owner(owner_id) }
}
