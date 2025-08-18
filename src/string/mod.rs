pub mod sanitize;
pub mod prompt;
pub mod styling;
pub mod boolean;
pub mod exclaim;
pub mod slug;
use std::collections::HashSet;

pub(crate) use slug::Sluggable;

pub(crate) type WordSet = HashSet<String>;
