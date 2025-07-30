use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{traits::tickable::Tickable, world::area::Area};
pub(crate) mod area;
pub(crate) mod room;

pub(crate) struct World {
    uptime: u64,
    areas: Vec<Area>,
}

pub type SharedWorld = Arc<Mutex<World>>;

impl World {
    pub(crate) fn new() -> Self {
        Self {
            uptime: 0,
            areas: vec![]
        }
    }
}

impl Tickable for World {
    fn tick(&mut self, uptime: u64) {
        self.uptime = uptime;
        for area in self.areas.iter_mut() {
            area.tick(self.uptime);
        }
    }
}
