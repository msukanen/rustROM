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
    pub fn is_builder(&self) -> bool {
        match self {
            Self::Admin   |
            Self::Builder => true,
            Self::Player {builder, ..} => *builder,
            _ => false
        }
    }

    pub fn is_event_host(&self) -> bool {
        match self {
            Self::Admin => true,
            Self::Player {event_host, ..} => *event_host,
            _ => false
        }
    }

    pub fn default() -> Self {
        Self::Player { builder: false, event_host: false }
    }
}
