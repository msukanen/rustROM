#[derive(Clone, Debug)]
pub enum BroadcastMessage {
    Say {
        room_id: String,
        message: String,
        from_player: String,
    },
    // TODO: add other types here later, e.g., Combat, Emote, etc.
}
