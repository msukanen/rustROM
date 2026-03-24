//! Some boolean related stuff…
pub trait AsSetting {
    fn as_state(&self) -> &'static str;
}

impl AsSetting for bool {
    fn as_state(&self) -> &'static str {
        match self {
            true => "set",
            _    => "uset"
        }
    }
}
