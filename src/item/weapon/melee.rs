use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::traits::{owned::{Owner, OwnerError, UNSPECIFIED_OWNER}, Identity, Owned};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MeleeInfo {
    id: String,
    owner: Owner,
}

impl Identity for MeleeInfo { fn id<'a>(&'a self) -> &'a str { &self.id }}

// TODO: Default is very likely temporary - but we'll use it for testing for now.
impl Default for MeleeInfo {
    fn default() -> Self {
        Self {
            id: Self::uuid(),
            owner: Owner::default(),
        }
    }
}

impl MeleeInfo {
    #[cfg(test)]
    pub(crate) fn re_id(&mut self) -> &mut Self {
        self.id = Self::uuid();
        self
    }

    fn uuid() -> String { format!("weapon-melee-{}", Uuid::new_v4())}
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