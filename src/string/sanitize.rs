pub trait Sanitizer {
    fn sanitize(&self) -> String;
}

impl Sanitizer for &str {
    fn sanitize(&self) -> String {
        self.chars()
            .filter(|c| !c.is_control())
            .collect::<String>()
    }
}

impl Sanitizer for String {
    fn sanitize(&self) -> String { self.as_str().sanitize()}
}

impl Sanitizer for &String {
    fn sanitize(&self) -> String { self.as_str().sanitize()}
}

pub(crate) fn clip_last_char<'a>(s: &'a str) -> &'a str {
    s.char_indices()
        .rev()
        .nth(0)
        .map(|(idx, _)| &s[..idx])
        .unwrap_or("")
}
