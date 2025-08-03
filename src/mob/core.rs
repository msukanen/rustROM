pub(crate) trait IsMob {
    /// Name of a mob.
    fn name<'a>(&'a self) -> &'a str;
}

pub(crate) struct MobCore {

}
