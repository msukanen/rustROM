pub mod clientstate;
pub(crate) use clientstate::ClientState;

pub mod contact;
pub mod direction;
pub mod password;

pub mod help;
pub(crate) use help::Help;

pub(crate) mod comm;
pub(crate) use comm::BroadcastMessage;

#[macro_export]
macro_rules! get_prompt {
    ($w:ident, $pt:expr, $default:tt) => {
        $w.read().await.prompts.get(&$pt).cloned().unwrap_or_else(|| $default.to_string())
    };
}
