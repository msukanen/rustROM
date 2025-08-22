use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::traits::{owned::UNSPECIFIED_OWNER, Identity, Owned};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MeleeInfo {
    id: String,
    owner: String,
}

impl Identity for MeleeInfo { fn id<'a>(&'a self) -> &'a str { &self.id }}

// TODO: Default is very likely temporary - but we'll use it for testing for now.
impl Default for MeleeInfo {
    fn default() -> Self {
        Self {
            id: Self::uuid(),
            owner: UNSPECIFIED_OWNER.into(),
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
    fn owner(&self) -> &str { &self.owner }
}

#[cfg(feature = "localtest")]
impl MeleeInfo {
    pub(crate) fn set_id(&mut self, id: &str) {
        self.id = id.into();
    }
}