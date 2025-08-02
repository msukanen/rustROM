use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub(crate) enum PromptType {
    Login,
    Password1, PasswordV,
    Playing,
    AFK,
    Custom(String)
}
