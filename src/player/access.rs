use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub(crate) enum Access {
    Dummy,
    Player {
        builder: bool,
        event_host: bool
    },
    Builder,
    Admin,
}

impl Access {
    /// Check if has builder rights.
    pub fn is_builder(&self) -> bool {
        match self {
            Self::Admin   |
            Self::Builder => true,
            Self::Player {builder, ..} => *builder,
            _ => false
        }
    }

    /// Check if is marked as an event host.
    pub fn is_event_host(&self) -> bool {
        match self {
            Self::Admin => true,
            Self::Player {event_host, ..} => *event_host,
            _ => false
        }
    }

    /// Get a clean slate default [Access::Player].
    pub fn default() -> Self {
        Self::Player { builder: false, event_host: false }
    }

    /// Check if has full admin rights.
    pub fn is_admin(&self) -> bool {
        match self {
            Self::Admin => true,
            _ => false
        }
    }
}
