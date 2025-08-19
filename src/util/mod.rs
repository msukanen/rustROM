pub(crate) mod clientstate;
pub(crate) use clientstate::ClientState;

pub(crate) mod contact;
pub(crate) mod direction;
pub(crate) mod password;

pub(crate) mod help;
pub(crate) use help::Help;

pub(crate) mod comm;
pub(crate) use comm::Broadcast;

pub mod badname;
pub mod github;
pub(crate) use github::GithubContent;

pub mod ed;
pub(crate) use ed::Editor;

#[macro_export]
macro_rules! get_prompt {
    ($w:ident, $pt:expr, $default:tt) => {
        $w.read().await.prompts.get(&$pt).cloned().unwrap_or_else(|| $default.to_string())
    };
}
