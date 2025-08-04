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
