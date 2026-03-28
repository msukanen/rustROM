use serde::{Deserialize, Serialize};

use crate::{item::{BlueprintID, blueprint::MELEE_BP_ID}, string::uuid_id::AsUuidId, traits::{Description, IdentityQuery, Owned, owned::{Owner, OwnerError}}};

// TODO: naming creativity!
fn title_default() -> String { "melee weapon".into() }
// TODO: description creativity!
fn desc_default() -> String { "a melee weapon of some sort".into() }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MeleeInfo {
    id: String,
    bp_id: String,
    #[serde(default)] owner: Owner,
    #[serde(default = "title_default")] title: String,
    #[serde(default = "desc_default")] description: String,
}

impl IdentityQuery for MeleeInfo {
    fn id<'a>(&'a self) -> &'a str { &self.id }
    fn title<'a>(&'a self) -> &'a str { &self.title }
}

// TODO: Default is very likely temporary - but we'll use it for testing for now.
impl Default for MeleeInfo {
    fn default() -> Self {
        Self {
            id: MELEE_BP_ID.uuided(),
            bp_id: MELEE_BP_ID.into(),
            owner: Owner::default(),
            title: title_default(),
            description: desc_default(),
        }
    }
}

impl MeleeInfo {
    #[cfg(test)]
    pub(crate) fn re_id(&mut self) -> &mut Self {
        self.id = MELEE_BP_ID.uuided();
        self
    }
}

impl Owned for MeleeInfo {
    fn owner(&self) -> &str { self.owner.owner() }
    fn original_owner(&self) -> &str { self.owner.original_owner() }
    fn set_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> { self.owner.set_owner(owner_id) }
    fn set_original_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> { self.owner.set_original_owner(owner_id) }
}

#[cfg(feature = "localtest")]
impl MeleeInfo {
    pub(crate) fn set_id(&mut self, id: &str) {
        self.id = id.into();
    }
}

impl Description for MeleeInfo {
    fn description<'a>(&'a self) -> &'a str { &self.description }
}

impl BlueprintID for MeleeInfo {
    fn bp_id(&self) -> &str { &self.bp_id }
}
