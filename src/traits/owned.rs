use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub(crate) static UNSPECIFIED_OWNER: &str = "";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Owner {
    curr_id: String,
    orig_id: String,
}

impl Default for Owner {
    fn default() -> Self {
        Self { curr_id: UNSPECIFIED_OWNER.into(), orig_id: UNSPECIFIED_OWNER.into() }
    }
}

impl Owned for Owner {
    fn owner(&self) -> &str { &self.curr_id }
    fn original_owner(&self) -> &str { &self.orig_id }
    // NOTE: It's up to the parent of Owner to check validity beforehand.
    fn set_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> {
        if owner_id == self.curr_id {
            return Err(OwnerError::AlreadySet);
        }
        self.curr_id = owner_id.to_string();
        Ok(())
    }
    // NOTE: It's up to the parent of Owner to check validity beforehand.
    fn set_original_owner(&mut self, owner_id: &str) -> Result<(), OwnerError> {
        if owner_id == self.orig_id {
            return Err(OwnerError::AlreadySet);
        }
        self.orig_id = owner_id.to_string();
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum OwnerError {
    NotOwnable,
    ImmutableOwnership,
    AlreadySet,
}

impl std::error::Error for OwnerError {}

impl Display for OwnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotOwnable => write!(f, "Cannot have an owner."),
            Self::ImmutableOwnership => write!(f, "Immutable ownership."),
            Self::AlreadySet => write!(f, "Ownership already set."),
        }
    }
}

pub(crate) trait Owned {
    /// Get current owner ID.
    fn owner(&self) -> &str;
    /// Get original owner ID.
    /// 
    /// Might or might not be same as [`Owned::owner`].
    fn original_owner(&self) -> &str;
    /// See if owned at all.
    fn is_owned(&self) -> bool { !self.owner().is_empty() }
    /// Set current owner ID.
    fn set_owner(&mut self, owner_id: &str) -> Result<(), OwnerError>;
    /// Set original owner ID.
    /// 
    /// NOTE: it's up to the very parent of final Owner instance to check validity of the claim.
    fn set_original_owner(&mut self, owner_id: &str) -> Result<(), OwnerError>;
}
