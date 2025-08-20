/// Various description-related functions…
pub trait Description {
    /// Title (printed name) of the entity.
    fn title<'a>(&'a self) -> &'a str;
    /// Description of the entity.
    fn description<'a>(&'a self) -> &'a str;
}

pub trait Identity {
    /// Name/ID of the entity.
    fn id<'a>(&'a self) -> &'a str;
}
