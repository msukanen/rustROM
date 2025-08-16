/// Various description-related functions…
pub trait Description {
    /// Name/ID of the entity.
    fn id<'a>(&'a self) -> &'a str;
    /// Title (printed name) of the entity.
    fn title<'a>(&'a self) -> &'a str;
    /// Description of the entity.
    fn description<'a>(&'a self) -> &'a str;
}
