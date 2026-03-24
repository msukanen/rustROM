
pub trait Identity {
    /// ID of the entity.
    fn id<'a>(&'a self) -> &'a str;
    /// Title/name of the entity.
    fn title<'a>(&'a self) -> &'a str;
}
