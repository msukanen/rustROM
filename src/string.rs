pub mod sanitize;
pub mod prompt;
pub mod styling;
pub mod boolean;
pub mod exclaim;

pub mod newline;
pub use newline::LineEndingExt;

pub mod slug;
pub use slug::Sluggable;

use std::collections::HashSet;
pub type WordSet = HashSet<String>;

pub mod unicode;
pub mod piglatin;
pub mod alpha;
pub mod robust_parse;
pub mod rx;
pub mod ansi;
pub mod uuid_id;
