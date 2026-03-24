use serde::{Deserialize, Serialize};

use crate::{item::BlueprintID, string::uuid_id::AsUuidId, traits::{Description, Identity, Owned, owned::{Owner, OwnerError}}};

// TODO: naming creativity!
fn title_default() -> String { "ranged weapon of some sort".into() }
// TODO: description creativity!
fn desc_default() -> String { "a melee weapon of some sort".into() }

const RANGED_BP_ID: &'static str = "weapon-ranged";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RangedInfo {
    id: String,
    bp_id: String,
    #[serde(default)] owner: Owner,
    #[serde(default = "title_default")] title: String,
    #[serde(default = "desc_default")] description: String,
}

impl Identity for RangedInfo {
    fn id<'a>(&'a self) -> &'a str { &self.id }
    fn title<'a>(&'a self) -> &'a str { &self.title }
}

// TODO: Default is very likely temporary - but we'll use it for testing for now.
impl Default for RangedInfo {
    fn default() -> Self {
        Self {
            id: RANGED_BP_ID.uuided(),
            bp_id: RANGED_BP_ID.into(),
            title: title_default(),
            owner: Owner::default(),
            description: desc_default(),
        }
    }
}

impl Owned for RangedInfo {
    fn owner(&self) -> &str { self.owner.owner() }
    fn original_owner(&self) -> &str { self.owner.original_owner() }
    fn set_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> { self.owner.set_owner(owner_id) }
    fn set_original_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> { self.owner.set_original_owner(owner_id) }
}

impl Description for RangedInfo {
    fn description<'a>(&'a self) -> &'a str { &self.description }
}

impl BlueprintID for RangedInfo {
    fn bp_id<'a>(&'a self) -> &'a str { &self.bp_id }
}
