//! What with...?

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref WHAT_WITH_ARG_RX: Regex = Regex::new(r#"\s*(?P<what>\S(?:.+?\S)?)\s+with\s+(?P<with>.+?)\s*$"#).unwrap();
}
