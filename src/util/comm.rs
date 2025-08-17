use crate::cmd::say::Subtype;

#[derive(Clone, Debug)]
pub enum BroadcastMessage {
    Say {
        subtype: Option<Subtype>,
        room_id: String,
        message: String,
        from_player: String,
    },
    // TODO: add other types here later, e.g., Combat, Emote, etc.
}
