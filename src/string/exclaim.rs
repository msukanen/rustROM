pub fn exclaim_if_needed<'a>(text: &'a str) -> String {
    match text.char_indices().rev().nth(0) {
        None |
        Some((_, '!')) => text.into(),
        Some((_, ch)) => if ch.is_alphanumeric() { format!("{}!", text)} else {text.into()},
    }
}
