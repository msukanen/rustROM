//! Exit states…

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::world::exit::{KeyError, jam::*};

pub const KEY_THAT_IS_NOT_A_KEY: &'static str = "!?$#<this is not a key>#$¿¡";

pub trait ExitStateQuery {
    /// Is the exit open?
    fn is_open(&self) -> bool;
    /// Check whether the exit can be opened (right now).
    fn can_open(&self) -> bool;

    /// Is the exit closed?
    fn is_closed(&self) -> bool;
    /// Check whether the exit can be closed.
    fn can_close(&self) -> bool;

    /// Is the exit locked?
    fn is_locked(&self) -> bool;
    /// Check whether the exit can be locked.
    /// 
    /// Unless the locking mechanism is jammed/broken, anything with a lock can be locked.
    fn can_lock(&self) -> bool;

    /// Is the exit jammed?
    fn is_jammed(&self) -> Option<JamState>;

    /// Get the key ID which (un)locks this exit.
    /// 
    /// # Returns
    /// Key ID, or a fake one if there is no key whatsoever.
    fn key_id<'a>(&'a self) -> &'a str { KEY_THAT_IS_NOT_A_KEY }
}

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
                        // autolock can't deduct jamming state (e.g. ExitState::Open doesn't have such),
                        // so we clone current one and hope for the best…
                        let method = jam.clone();
                        if let Self::Locked { jam,.. } = self {
                            *jam = method;
                        }
                        return Ok(true);
                    }
                    Ok(false)
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

    /// Force unlocked state. Any [jamming][JamState] is wiped.
    pub fn force_unlock(&mut self) {
        if let Err(KeyError::Jammed) = self.unlock_with(&self.key_id().to_string()) {
            *self = Self::Closed { key_id: self.key_id().to_string().into(), jam: None }
        }
    }

    /// Jam the state, if it can be done.
    /// 
    /// # Args
    /// - `how` to jam it.
    /// 
    /// # Returns
    /// `true` if actually jammed.
    pub fn jam(&mut self, how: JamState) -> bool {
        match self {
            Self::Open { key_id } |
            Self::Closed { key_id, jam: None } => {*self = Self::Closed { key_id: key_id.clone(), jam: Some(how) }; true},
            Self::Locked { key_id, jam: None } => {*self = Self::Locked { key_id: key_id.clone(), jam: Some(how) }; true},
            _ => false
        }
    }
}

/// Equalize state of the opposite end of `$exit`.
/// 
/// # Args
/// - `$equalize`— actually equalize?
/// - `$ctx`…
/// - `$r_id`— current [Room][crate::world::room::Room] ID.
/// - `$exit`— [Exit][crate::world::exit::Exit] which state to clone.
#[macro_export]
macro_rules! equalize_opposite_exit_state {
    ($equalize:ident, $ctx:ident, $r_id:ident, $exit:ident) => {
        if $equalize && $exit.is_some() && $r_id.is_some() {
            let exit = $exit.unwrap();
            let r_id = $r_id.unwrap();
            let mut wlock = $ctx.world.write().await;
            if let Some(oroom) = wlock.rooms.get_mut(&exit.destination) {
                let mut orlock = oroom.write().await;
                for oexit in orlock.exits.values_mut() {
                    if oexit.state == crate::world::exit::state::ExitState::AlwaysOpen {
                        continue;
                    }

                    if oexit.destination == r_id {
                        oexit.state = exit.state.clone()
                    }
                }
            }
        }
    };
}

impl ExitStateQuery for ExitState {
    fn is_open(&self) -> bool {
        match self {
            Self::AlwaysOpen |
            Self::Open {..}  => true,
            _ => false
        }
    }

    fn can_open(&self) -> bool {
        match self {
            Self::AlwaysOpen  |
            Self::Open { .. } |
            Self::Closed { jam: Some(_),.. } |
            Self::Locked { jam: Some(_),.. } => false,
            _ => true
        }
    }

    fn can_close(&self) -> bool {
        !matches!(self, Self::AlwaysOpen)
    }

    #[inline]
    fn is_closed(&self) -> bool { !self.is_open() }

    fn can_lock(&self) -> bool {
        if !self.can_close() { return false; }

        match self {
            Self::Open { key_id: Some(_) }  |
            Self::Closed { key_id: Some(_),.. }|
            Self::Locked { jam: None,..} => true,
            _ => false
        }
    }

    fn is_locked(&self) -> bool {
        matches!(self, Self::Locked {..})
    }

    fn is_jammed(&self) -> Option<JamState> {
        match self {
            Self::AlwaysOpen |
            Self::Open {..}  => None,
            Self::Closed {jam,..} |
            Self::Locked {jam,..} => *jam
        }
    }

    fn key_id<'a>(&'a self) -> &'a str {
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
}
