//! # Item Blueprints.
//! 
//! Items generally are based on blueprints when created.
//!
//! These blueprints may be custom in the core, but they all
//! fall into a few general categories.

use serde::{Deserialize, Serialize};

use crate::traits::IdentityQuery;

/// A trait for anything and everything with a "blueprint" of sorts.
pub trait BlueprintID : IdentityQuery {
    /// Get the blueprint ID.
    //
    // Blueprint ID ≠ ID, except in case of truly unique items.
    //
    fn bp_id<'a>(&'a self) -> &'a str;
}

pub(crate) const BACKPACK_BP_ID: &'static str = "backpack";
pub(crate) const RANGED_BP_ID: &'static str = "weapon-ranged";
pub(crate) const MELEE_BP_ID: &'static str = "weapon-melee";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Blueprint {
    id: String,
    title: String,
}

impl BlueprintID for Blueprint {
    fn bp_id<'a>(&'a self) -> &'a str { &self.id }
}

impl IdentityQuery for Blueprint {
    fn id<'a>(&'a self) -> &'a str { self.bp_id() }
    fn title<'a>(&'a self) -> &'a str { &self.title }
}
