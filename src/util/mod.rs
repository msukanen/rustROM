pub mod contact;
pub mod direction;
pub mod password;

#[macro_export]
macro_rules! get_prompt {
    ($w:ident, $pt:expr, $default:tt) => {
        $w.read().await.prompts.get(&$pt).cloned().unwrap_or_else(|| $default.to_string())
    };
}
