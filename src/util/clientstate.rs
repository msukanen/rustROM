#[derive(Debug, Clone, PartialEq, Eq)]
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

impl PartialEq for ClientState {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::EnteringName => match other { Self::EnteringName => true,_=> false },
            Self::EnteringPassword1 { .. } => match other { Self::EnteringPassword1 { .. } => true,_=> false },
            Self::EnteringPasswordV { .. } => match other { Self::EnteringPasswordV { .. } => true,_=> false },
            Self::Playing => match other { Self::Playing => true,_=> false },
            Self::Logout => match other { Self::Logout => true,_=> false },
            Self::Editing { mode } => {
                let mode1 = mode;
                match other {
                    Self::Editing { mode } => *mode1 == *mode,
                    _ => false
                }
            }
        }
    }
}
