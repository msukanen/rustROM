//! Jamming of doors, latches, locks, etc.

use serde::{Deserialize, Serialize};
/// Methods of jamming.
// No jam here though, berry or otherwise.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum JammingMethod {
    Sabotaged,
    Barred,
    Welded,
    Vault
}

/// State of jam.
// Edible isn't one of the options…
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum JamState {
    Ok,
    LockOnly(JammingMethod),
    WholeExit(JammingMethod),
}

impl Default for JamState {
    fn default() -> Self {
        Self::Ok
    }
}
