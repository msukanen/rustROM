//! Keys!

use serde::{Deserialize, Serialize};

use crate::{item::BlueprintID, traits::{Description, IdentityQuery, Owned, owned::{Owner, OwnerError}}};

// TODO: naming creativity!.
fn title_default() -> String { "a key".into() }

/// It's a me, a key!
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Key {
    id: String,
    bp_id: String,
    #[serde(default)] owner: Owner,
    #[serde(default = "title_default")] title: String,
    #[serde(default = "title_default")] description: String,// = title_default…
    #[serde(default)] pub one_time: bool,
}

impl IdentityQuery for Key {
    fn id<'a>(&'a self) -> &'a str { &self.id }
    fn title<'a>(&'a self) -> &'a str { &self.title }
}

impl Key {
    /// Create a new [Key].
    /// 
    /// Unsurprisingly, [keys][Key] are used for (un)locking (un)lockable things.
    /// Each lock is (or should be…) keyed to specific [BP#ID][BlueprintID].
    /// 
    /// # Args
    /// - `id_stem`— ID-stem from which UUID loaded ID and the [Key]'s [BP#ID][BlueprintID] will be derived from.
    /// - `one_time` use [Key]?
    pub fn new(id_stem: &str, one_time: bool) -> Self {
        use crate::string::uuid_id::AsUuidId;

        Self {
            id: id_stem.uuided(),
            bp_id: id_stem.into(),
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

impl BlueprintID for Key {
    fn bp_id<'a>(&'a self) -> &'a str { &self.bp_id }
}
