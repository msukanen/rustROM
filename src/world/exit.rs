//! [Exit], a way out of e.g. a [Room].
use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ExitState {
    Open,
    Closed,
    Locked { key_id: String }
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
        ExitState::Open
    }
}

impl Exit {
    pub fn is_closed(&self) -> bool {
        matches!(self.state, ExitState::Closed)
    }
}
