/// See if `c` is a vocal.
/// 
/// What is a 'vocal' and what is a 'consonant' varies somewhat between
/// languages, but (as of writing) we'll be using a quite generic (= English)
/// approach to them.
pub fn is_a_vocal(c: char) -> bool {
    match c.to_ascii_lowercase() {
        'a'|'e'|'i'|'o'|'u' => true,
        _ => false
    }
}

/// See if the optional `c` is a vocal.
pub fn maybe_a_vocal(c: Option<char>) -> bool {
    let Some(c) = c else { return false };
    is_a_vocal(c)
}
