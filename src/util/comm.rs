use crate::{cmd::say::Subtype, player::Player};

/// Various global channel types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Channel {
    /// Channel for questions & answers.
    Qa,
    /// Channel for newbie questions, talk, help, etc.
    Newbie,
    /// OOC noise …
    Ooc,
    /// Builder-only channel.
    Builder,
    /// Admin-only channel.
    Admin,
    /// Event coordination.
    Event,
}

impl Channel {
    /// Check if the player has permission to listen to this channel.
    pub fn can_listen(&self, player: &Player) -> bool {
        match self {
            Self::Admin   => player.access.is_admin(),
            Self::Builder => player.access.is_builder(),
            Self::Event   => player.access.is_event_host(),
            Self::Newbie  |
            Self::Ooc     |
            Self::Qa      => true
        }
    }
}

/// Various broadcast types.
#[derive(Clone, Debug)]
pub enum Broadcast {
    Say {
        subtype: Option<Subtype>,
        room_id: String,
        message: String,
        from_player: String,
    },
    Shout {
        room_id: String,
        message: String,
        from_player: String,
    },
    Tell {
        subtype: Option<Subtype>,
        message: String,
        to_player: String,
        from_player: String,
    },
    /// Special system/admin broadcast.
    Force {
        message: String,
        to_player: Option<String>,
        from_player: Option<String>,
    },
    Channel {
        channel: Channel,
        message: String,
        from_player: String,
    }
}

/* pub(crate) trait MessagePayload {
    fn message<'a>(&'a self) -> &'a str;
    fn from_player(&self) -> String;
}

impl MessagePayload for Broadcast {
    fn from_player(&self) -> String {
        match self {
            Self::Channel { from_player, .. }|
            Self::Shout { from_player , ..}|
            Self::Say { from_player, .. }|
            Self::Tell { from_player , ..} => from_player.clone(),
            Self::Force { from_player, .. } => from_player.clone().unwrap_or("".into()),
        }
    }

    fn message<'a>(&'a self) -> &'a str {
        match self {
            Self::Channel { message, .. }|
            Self::Force { message, .. }|
            Self::Say { message, .. }|
            Self::Shout { message, .. }|
            Self::Tell { message, .. } => &message
        }
    }
}
 */