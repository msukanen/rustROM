use crate::cmd::say::Subtype;

#[derive(Clone, Debug)]
pub enum Message {

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
    }
}

pub(crate) trait MessagePayload {
    fn message<'a>(&'a self) -> &'a str;
    fn from_player(&self) -> String;
}

impl MessagePayload for Broadcast {
    fn from_player(&self) -> String {
        match self {
            Self::Shout { from_player , ..}|
            Self::Say { from_player, .. }|
            Self::Tell { from_player , ..} => from_player.clone(),
            Self::Force { from_player, .. } => from_player.clone().unwrap_or("".into()),
        }
    }

    fn message<'a>(&'a self) -> &'a str {
        match self {
            Self::Force { message, .. }|
            Self::Say { message, .. }|
            Self::Shout { message, .. }|
            Self::Tell { message, .. } => &message
        }
    }
}
