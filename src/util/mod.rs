pub(crate) mod contact;

#[macro_export]
macro_rules! tell_user {
    ($w:ident, $topic:expr) => {
        $w.write_all($topic.as_bytes()).await.unwrap();
    };
}

#[macro_export]
macro_rules! tell_user_p {
    ($w:ident, $prompt:expr, $topic:expr) => {
        $w.write_all(format!("{}\n\n{}", $topic, $prompt).as_bytes()).await.unwrap();
    };
}

#[macro_export]
macro_rules! get_g_prompt {
    ($guard:expr, $pt:expr, $default:tt) => {
        $guard.prompts.get(&$pt).cloned().unwrap_or_else(|| $default.to_string())
    };
}

#[macro_export]
macro_rules! get_prompt {
    ($w:ident, $pt:expr, $default:tt) => {
        get_g_prompt!($w.read().await, $pt, $default)
    };
}
