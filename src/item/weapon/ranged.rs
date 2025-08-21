use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::traits::Identity;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RangedInfo {
    id: String,
}

impl Identity for RangedInfo { fn id<'a>(&'a self) -> &'a str { &self.id }}

// TODO: Default is very likely temporary - but we'll use it for testing for now.
impl Default for RangedInfo {
    fn default() -> Self {
        Self {
            id: format!("weapon-ranged-{}", Uuid::new_v4()),
        }
    }
}
