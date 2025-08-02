pub(crate) mod sanitize;
pub(crate) mod prompt;
/// Replaces any non-alphabetic characters in a string with underscores.
///
/// This is useful for creating safe(ish), simple filenames from user input
/// like player names.  For example, "Super*Star!" becomes "Super_Star_".
///
/// # Arguments
/// - `input`— The raw string to sanitize.
///
/// # Returns
/// A new `String` that is safe to use as a filename component.
pub fn slugify(input: &str) -> String {
    input.chars()
        .map(|c| {
            if c.is_ascii_alphabetic() { c }
            else { '_' }
        })
        .collect()
}

pub trait Sluggable {
    fn slugify(&self) -> String;
}

impl Sluggable for String { fn slugify(&self) -> String { slugify(self) }}
impl Sluggable for &String { fn slugify(&self) -> String { slugify(self) }}
impl Sluggable for &str { fn slugify(&self) -> String { slugify(self) }}
