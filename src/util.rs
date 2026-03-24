pub mod clientstate;
pub use clientstate::ClientState;

pub mod contact;
pub mod direction;
pub mod password;

pub mod help;
pub use help::Help;

pub mod comm;
pub use comm::Broadcast;

pub mod badname;
mod github;
pub use github::GithubContent;

pub mod ed;
pub use ed::Editor;

pub mod boolean;
pub use boolean::AsSetting;

#[macro_export]
macro_rules! get_prompt {
    ($w:ident, $pt:expr, $default:tt) => {
        $w.read().await.prompts.get(&$pt).cloned().unwrap_or_else(|| $default.to_string())
    };
}
