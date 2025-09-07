pub mod sanitize;
pub mod prompt;
pub mod styling;
pub mod boolean;
pub mod exclaim;

pub mod newline;
pub(crate) use newline::LineEndingExt;

pub mod slug;
pub(crate) use slug::Sluggable;

use std::collections::HashSet;
pub(crate) type WordSet = HashSet<String>;

pub(crate) mod unicode;

pub(crate) mod piglatin;
pub(crate) mod alpha;
