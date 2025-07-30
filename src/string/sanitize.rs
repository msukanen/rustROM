/// Sanitize input by removing all e.g. Telnet control characters, etc.
pub(crate) fn sanitize_input(input: &str) -> String {
    input.chars().map(|c| c.is_ascii_alphanumeric() || c.is_whitespace()).collect()
}
