use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::traits::Identity;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MeleeInfo {
    id: String
}

impl Identity for MeleeInfo { fn id<'a>(&'a self) -> &'a str { &self.id }}

// TODO: Default is very likely temporary - but we'll use it for testing for now.
impl Default for MeleeInfo {
    fn default() -> Self {
        Self {
            id: format!("weapon-melee-{}", Uuid::new_v4()),
        }
    }
}
