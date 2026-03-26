//! Exit states…

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::world::exit::{KeyError, jam::*};

pub const KEY_THAT_IS_NOT_A_KEY: &'static str = "!?$#<this is not a key>#$¿¡";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ExitState {
    AlwaysOpen,
    Open { key_id: Option<String> },
    Closed { key_id: Option<String>, jam: Option<JamState> },
    Locked { key_id: String, jam: Option<JamState> }
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
    /// Get the key ID which (un)locks this [exit][ExitState].
    /// 
    /// # Returns
    /// Key ID, or a fake one if there is no key whatsoever.
    pub fn key_id<'a>(&'a self) -> &'a str {
        match self {
            Self::AlwaysOpen => KEY_THAT_IS_NOT_A_KEY,

            Self::Open { key_id } |
            Self::Closed { key_id,..} =>
                match key_id {
                    None => KEY_THAT_IS_NOT_A_KEY,
                    Some(id) => id
                },
            
            Self::Locked { key_id,..} => &key_id
        }
    }

    /// Open the [exit][ExitState], if possible.
    /// 
    /// # Returns
    /// `true` if state changed.
    pub fn open(&mut self) -> bool {
        match self {
            Self::AlwaysOpen    |
            Self::Open {..}     |
            Self::Locked { .. } => false,
            Self::Closed { key_id, jam: None } => { *self = Self::Open { key_id: key_id.clone() }; true },
            _                   => false
        }
    }

    /// Close the [exit][ExitState], if possible.
    /// 
    /// # Returns
    /// `true` if state changed.
    pub fn close(&mut self) -> bool {
        match self {
            Self::Open { key_id } => { *self = Self::Closed { key_id: key_id.clone(), jam: None }; true },
            _ => false
        }
    }

    /// Check if the exit is closed.
    pub fn is_closed(&self) -> bool {
        match self {
            Self::AlwaysOpen  |
            Self::Open { .. } => false,
            _ => true
        }
    }

    /// Lock the exit if possible.
    /// 
    /// Note that only *unjammed* exits can be locked.
    /// 
    /// # Returns
    /// `true` if state changed.
    fn autolock(&mut self) -> bool {
        match self {
            Self::Open { key_id: Some(key_id) }   |
            Self::Closed { key_id: Some(key_id), jam: None } => { *self = Self::Locked { key_id: key_id.clone(), jam: None }; true },
            _ => false
        }
    }

    /// Lock the exit with `key_id`.
    /// 
    /// # Returns
    /// `true` if state changed.
    pub fn lock_with(&mut self, key_id: &str) -> Result<bool, KeyError> {
        // self.clown()ing due borrow checker…
        match self.clone() {
            Self::AlwaysOpen => Err(KeyError::NotLockable),
            Self::Open { key_id: Some(id) }
                => if id == key_id { Ok(self.autolock()) } else { Err(KeyError::IncorrectKey)},
            Self::Closed { key_id: Some(id), jam }
                => {
                    if id == key_id && self.autolock() {
                        // autolock can't deduct 
                        let method = jam.clone();
                        if let Self::Locked { jam,.. } = self {
                            *jam = method;
                        }
                    }
                    Ok(true)
                }
            _ => Ok(false)
        }
    }

    /// Unlock the exit with specified key, if possible, and leave it (when applicable) at [closed][ExitState::Closed] state.
    /// 
    /// # Returns
    /// `true` if state changed.
    pub fn unlock_with(&mut self, with_key_id: &str) -> Result<bool, KeyError> {
        match self {
            Self::Locked { key_id, jam: None } =>
                if key_id == with_key_id {
                    *self = Self::Closed { key_id: Some(key_id.clone()), jam: None };
                    Ok(true)
                } else {
                    Err(KeyError::IncorrectKey)
                },
            Self::Locked {..} => Err(KeyError::Jammed),
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
            Self::Closed { key_id: Some(_),.. }|
            Self::Locked { jam: None,..} => true,
            _ => false
        }
    }
}

/// Equalize state of the opposite end of `$exit`.
/// 
/// # Args
/// - `$ctx`…
/// - `$r_id`— current [Room][crate::world::room::Room] ID.
/// - `$exit`— [Exit][crate::world::exit::Exit] which state to clone.
#[macro_export]
macro_rules! equalize_opposite_exit_state {
    ($ctx:ident, $r_id:ident, $exit:ident) => {{
        let mut wlock = $ctx.world.write().await;
        if let Some(oroom) = wlock.rooms.get_mut(&$exit.destination) {
            let mut orlock = oroom.write().await;
            for oexit in orlock.exits.values_mut() {
                if oexit.state == crate::world::exit::state::ExitState::AlwaysOpen {
                    continue;
                }

                if oexit.destination == $r_id {
                    oexit.state = $exit.state.clone()
                }
            }
        }
    }};
}
