use std::fmt::Display;

use unicode_normalization::UnicodeNormalization;

#[derive(Debug, PartialEq, Clone)]
pub enum IdError {
    /// Input was entirely non-alphanum (or empty).
    EmptyOrGarbage,
    /// Input too long for filesystem (OS limits).
    TooLong,
    /// Input contains forbidden/reserved (e.g. any of the hardcoded bootstrap) patterns.
    ReservedName(String),
}

impl Display for IdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyOrGarbage => write!(f, "Well, identity contains no readable alphanum characters. Might want to rethink that…"),
            Self::ReservedName(n) => write!(f, "Sorry, but '{n}' is already reserved by the system itself…"),
            Self::TooLong => write!(f, "OS said that we can't use that long string for identity. Squeeze a bit…"),
        }
    }
}

/// Ensure `input` sanity for file system and e.g. [Identity][crate::Identity] purposes.
/// 
/// # Args
/// - `input` to be sanitized.
pub(crate) fn as_id(input: &str) -> Result<String, IdError> {
    let mut out = String::new();
    let mut last_was_junk = false;
    let mut has_alnum = false;

    for c in input.trim().nfd() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_was_junk = false;
            has_alnum = true;
            continue;
        }

        if !last_was_junk && !out.is_empty() {
            out.push(match c {
                '-' |
                _ if c.is_whitespace() => '-',
                _ => '_'
            });
            last_was_junk = true
        }
    }

    if !has_alnum || out.is_empty() {
        return Err(IdError::EmptyOrGarbage);
    }

    if out.len() > 255 {
        return Err(IdError::TooLong);
    }

    Ok(out)
}

/// Ensure `input` sanity for [Player][crate::Player] identifying and file naming.
/// 
/// # Args
/// - `input` to be sanitized.
pub(crate) fn slugify(input: &str) -> String {
    input.chars()
        .map(|c| {
            if c.is_ascii_alphabetic() || c == '-' { c }
            else { '_' }
        })
        .collect()
}

pub trait Sluggable {
    fn slugify(&self) -> String;
    fn as_id(&self) -> Result<String, IdError>;
}

impl Sluggable for String {
    #[inline] fn slugify(&self) -> String { slugify(self)}
    #[inline] fn as_id(&self) -> Result<String, IdError> { as_id(self)}
}
impl Sluggable for &String {
    #[inline] fn slugify(&self) -> String { slugify(self)}
    #[inline] fn as_id(&self) -> Result<String, IdError> { as_id(self)}
}
impl Sluggable for &str {
    #[inline] fn slugify(&self) -> String { slugify(self)}
    #[inline] fn as_id(&self) -> Result<String, IdError> { as_id(self)}
}

#[cfg(test)]
mod slug_tests {
    use unicode_normalization::UnicodeNormalization;

    #[test]
    fn as_id() {
        let src = "Ali  bab ---atsuu";
        let out = src
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
        assert_ne!(src, out.as_str());
        assert_eq!("Ali-bab-atsuu", out.as_str());
    }

    #[test]
    fn as_id_2() {
        let src = "blob#!!#$$2";
        let mut out = String::new();
        let mut last_was_junk = false;
        for c in src.trim().nfd() {
            if c.is_ascii_alphanumeric() {
                out.push(c.to_ascii_lowercase());
                last_was_junk = false;
                continue;
            }

            if !last_was_junk && !out.is_empty() {
                out.push(match c {
                    '-' |
                    _ if c.is_whitespace() => '-',
                    _ => '_'
                });
                last_was_junk = true
            }
        }

        assert_ne!("blob2", out.as_str());
        assert_eq!("blob_2", out.as_str());
    }
}
