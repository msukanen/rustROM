pub trait BooleanCheckExt {
    /// Value is considered 'true'?
    fn is_true(&self) -> bool;
    /// Value is considered 'boolean'?
    fn is_boolean(&self) -> bool;
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

    fn is_boolean(&self) -> bool {
        if self.is_true() { return true;}
        match *self {
            "false" |
            "unset" |
            "off"   => true,
            _       => false
        }
    }
}
