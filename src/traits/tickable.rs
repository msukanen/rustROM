/// A trait for anything that ticks.
pub(crate) trait Tickable {
    fn tick(&mut self, uptime: u64);
}
