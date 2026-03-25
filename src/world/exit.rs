//! [Exit], a way out of e.g. a [Room].

use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// All sorts of key related errors…
#[derive(Debug, Clone, Copy)]
pub enum KeyError {
    /// Key wasn't correct, obviously.
    IncorrectKey,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ExitState {
    AlwaysOpen,
    Open { key_id: Option<String> },
    Closed { key_id: Option<String> },
    Locked { key_id: String }
}

impl Display for ExitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Closed{..} => "closed",
            Self::AlwaysOpen |
            Self::Open{..}   => "open",
            Self::Locked{..} => "locked tight"
        })
    }
}

impl ExitState {
    /// Open the exit, if possible.
    /// 
    /// # Returns
    /// `true` if state changed.
    pub fn open(&mut self) -> bool {
        match self {
            Self::AlwaysOpen    |
            Self::Open {..}     |
            Self::Locked { .. } => false,
            Self::Closed { key_id } => { *self = Self::Open { key_id: key_id.clone() }; true },
        }
    }

    /// Close the exit, if possible.
    /// 
    /// # Returns
    /// `true` if state changed.
    pub fn close(&mut self) -> bool {
        match self {
            Self::Open { key_id } => { *self = Self::Closed { key_id: key_id.clone() }; true },
            _ => false
        }
    }

    /// Lock the exit if possible.
    /// 
    /// # Returns
    /// `true` if state changed.
    fn autolock(&mut self) -> bool {
        match self {
            Self::Open { key_id: Some(key_id) }   |
            Self::Closed { key_id: Some(key_id) } => { *self = Self::Locked { key_id: key_id.clone() }; true },
            _ => false
        }
    }

    /// Lock the exit with `key_id`.
    /// 
    /// # Returns
    /// `true` if state changed.
    pub fn lock_with(&mut self, key_id: &str) -> Result<bool, KeyError> {
        match self {
            Self::Closed { key_id: Some(lock_key_id) } |
            Self::Open { key_id: Some(lock_key_id) }   =>
                if lock_key_id == key_id {
                    Ok(self.autolock())
                } else {
                    Err(KeyError::IncorrectKey)
                },
            _ => Ok(false)
        }
    }

    /// Unlock the exit with specified key, if possible, and leave it (when applicable) at [closed][ExitState::Closed] state.
    /// 
    /// # Returns
    /// `true` if state changed.
    pub fn unlock_with(&mut self, with_key_id: &str) -> Result<bool, KeyError> {
        match self {
            Self::Locked { key_id } =>
                if key_id == with_key_id {
                    *self = Self::Closed { key_id: Some(key_id.clone()) };
                    Ok(true)
                } else {
                    Err(KeyError::IncorrectKey)
                },
            _ => Ok(false)
        }
    }

    /// Check whether the exit can be closed.
    pub fn can_close(&self) -> bool {
        !matches!(self, Self::AlwaysOpen)
    }

    /// Check whether the exit can be locked.
    pub fn can_lock(&self) -> bool {
        if !self.can_close() { return false; }

        match self {
            Self::Open { key_id: Some(_) }  |
            Self::Closed { key_id: Some(_) }|
            Self::Locked {..} => true,
            _ => false
        }
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
