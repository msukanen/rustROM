/// Various description-related functions…
pub trait Description {
    /// Description of the entity.
    fn description<'a>(&'a self) -> &'a str;
}
