#[derive(Debug, Clone)]
pub enum EditorMode {
    Room { id: String },
    Help { topic: String },
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

impl EditorMode {
    pub(crate) fn get(&self, func_name: &str) -> Option<String> {
        match self {
            Self::Help { .. } => match func_name {
                "save" => Some("hedit-save".into()),
                "exit" => Some("hedit-exit".into()),
                _ => None
            },

            Self::Room { .. } => match func_name {
                "save" => Some("redit-save".into()),
                "exit" => Some("redit-exit".into()),
                _ => None
            }
        }
    }
}
