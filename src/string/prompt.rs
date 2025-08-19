use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum PromptType {
    Login,
    Password1, PasswordV,
    Playing,
    AFK,
    Custom(String)
}

#[macro_export]
macro_rules! tell_user {
    ($w:expr, $t:expr) => {
        tokio::io::AsyncWriteExt::write_all($w, crate::string::styling::format_color($t).as_bytes()).await.unwrap()
    };

    ($w:expr, $fmt:literal, $($arg:tt)*) => {{
        let msg = format!($fmt, $($arg)*);
        tell_user!($w, &msg);
    }}
}

#[macro_export]
macro_rules! tell_user_unk {
    ($w:expr) => {
        crate::tell_user!($w, "Huh?\n")
    };
}
