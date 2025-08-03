/// A trait for anything that ticks.
pub(crate) trait Tickable {
    /// Tick along, tick along …
    fn tick(&mut self, uptime: u64);
}
