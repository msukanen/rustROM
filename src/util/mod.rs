pub mod contact;
pub mod direction;
pub mod password;

//#[macro_export] macro_rules! tell_user {    ($w:expr, $($arg:tt)*) => {        $w.write_all(format!($($arg)*).as_bytes()).await.unwrap();    };}

//#[macro_export] macro_rules! prompt_user {    ($ctx:expr) => {        tell_user!($ctx.writer, "{}", $ctx.prompt);    };}

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
