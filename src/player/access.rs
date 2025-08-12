use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum Access {
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

#[macro_export]
macro_rules! validate_builder {
    ($ctx:expr) => {
        if !$ctx.player.read().await.access.is_builder() {
            crate::tell_user_unk!($ctx.writer);
            crate::resume_game!($ctx);
        }
    };
}

#[macro_export]
macro_rules! validate_admin {
    ($ctx:expr) => {
        if !$ctx.player.read().await.access.is_admin() {
            crate::tell_user_unk!($ctx.writer);
            crate::resume_game!($ctx);
        }
    };
}
