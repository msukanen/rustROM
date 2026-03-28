//! Tools core…

use serde::{Deserialize, Serialize};

use crate::{item::BlueprintID, traits::{IdentityQuery, Owned, owned::*}};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tool {
    id: String,
    title: String,
    #[serde(default)]
    owner: Owner,
    bp_id: String,
    pub single_use: bool,
}

impl BlueprintID for Tool {
    fn bp_id<'a>(&'a self) -> &'a str { &self.bp_id }
}

impl IdentityQuery for Tool {
    fn id<'a>(&'a self) -> &'a str { &self.id }
    fn title<'a>(&'a self) -> &'a str { &self.title }
}

impl Owned for Tool {
    fn owner(&self) -> &str { self.owner.owner() }
    fn original_owner(&self) -> &str { self.owner.original_owner() }
    fn set_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> { self.owner.set_owner(owner_id) }
    fn set_original_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> { self.owner.set_original_owner(owner_id) }
}
