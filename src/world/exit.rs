//! [Exit], a way out of e.g. a [Room].

use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub mod state;
pub mod jam;
use state::*;

use crate::world::exit::jam::JamState;

/// All sorts of key related errors…
#[derive(Debug, Clone, Copy)]
pub enum KeyError {
    /// Not lockable, at all.
    NotLockable,
    /// Key wasn't correct, obviously.
    IncorrectKey,
    /// Jammed…
    Jammed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Exit {
    pub destination: String,
    #[serde(default)]
    pub state: ExitState,
}

impl PartialEq for Exit {
    fn eq(&self, other: &Self) -> bool {
        self.destination == other.destination
    }
}

impl Display for Exit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.destination)
    }
}

impl From<&str> for Exit {
    fn from(destination: &str) -> Self {
        Self {
            destination: destination.into(),
            state: ExitState::default(),
        }
    }
}

// Just a "lazy convenience" From<>…
impl From<&&str> for Exit {
    fn from(value: &&str) -> Self {
        (*value).into()
    }
}

impl Default for ExitState {
    fn default() -> Self {
        ExitState::Open { key_id: None }
    }
}

impl ExitStateQuery for Exit {
    fn can_close(&self) -> bool { self.state.can_close()}
    fn is_closed(&self) -> bool { self.state.is_closed()}
    fn can_lock(&self) -> bool { self.state.can_lock()}
    fn can_open(&self) -> bool { self.state.can_open()}
    fn is_jammed(&self) -> Option<JamState> { self.state.is_jammed()}
    fn is_locked(&self) -> bool { self.state.is_locked()}
    fn is_open(&self) -> bool { self.state.is_open()}
    fn key_id(&self) -> &str { self.state.key_id()}
}

#[cfg(test)]
mod room_exit_tests {
    use super::*;

    #[test]
    fn exit_serde_deser() {
        let _ = env_logger::try_init();
        let exit = Exit {
            destination: "nowhere-much".into(),
            state: ExitState::Open { key_id: None }
        };
        let json = serde_json::to_string_pretty(&exit).unwrap();
        log::debug!("JSON: {json}");
    }
}
