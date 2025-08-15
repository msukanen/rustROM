pub trait BooleanCheckExt {
    /// Value is considered 'true'?
    fn is_true(&self) -> bool;
}

impl BooleanCheckExt for &str {
    fn is_true(&self) -> bool {
        if self.starts_with('+') { return true; }
        match *self {
            "true" |
            "set"  |
            "on"   => true,
            _      => false
        }
    }
}
