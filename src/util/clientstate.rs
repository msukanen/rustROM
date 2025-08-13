#[derive(Debug, Clone)]
pub enum EditorMode {
    Room,
    Help,
}

#[derive(Debug, Clone)]
pub enum ClientState {
    EnteringName,
    EnteringPassword1 { name: String },
    EnteringPasswordV { name: String, pw1: String },
    Playing,
    Editing { mode: EditorMode },
    Logout,
}
