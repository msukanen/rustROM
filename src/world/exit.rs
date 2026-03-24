//! [Exit], a way out of e.g. a [Room].
use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ExitState {
    Open { key_id: Option<String> },
    Closed { key_id: Option<String> },
    Locked { key_id: String }
}

impl Display for ExitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Closed{..} => "closed",
            Self::Open{..} => "open",
            Self::Locked{..} => "locked tight"
        })
    }
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

impl Exit {
    pub fn is_closed(&self) -> bool {
        !matches!(self.state, ExitState::Open{..})
    }
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
