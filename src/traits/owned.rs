pub(crate) static UNSPECIFIED_OWNER: &str = "";

pub(crate) trait Owned {
    fn owner(&self) -> &str;
    fn is_owned(&self) -> bool { !self.owner().is_empty() }
}
