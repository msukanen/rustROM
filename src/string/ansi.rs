//! Some ANSI related functionality.
use std::borrow::Cow; // is there a Bull too?

use lazy_static::lazy_static;

lazy_static! {
    static ref ANSI_RX: regex::Regex = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
}

pub trait AntiAnsi {
    fn strip_ansi(&self) -> Cow<'_, str>;
}

impl AntiAnsi for String {
    fn strip_ansi(&self) -> Cow<'_, str> {
        (*ANSI_RX).replace_all(self, "")
    }
}
