
pub trait Identity {
    /// Name/ID of the entity.
    fn id<'a>(&'a self) -> &'a str;
}
