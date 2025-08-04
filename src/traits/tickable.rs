use async_trait::async_trait;

/// A trait for anything that ticks.
#[async_trait]
pub trait Tickable {
    /// Tick along, tick along …
    async fn tick(&mut self, uptime: u64);
}
