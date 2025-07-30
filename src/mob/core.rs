pub(crate) trait IsMob {
    fn name<'a>(&'a self) -> &'a str;
}

pub(crate) struct MobCore {

}
