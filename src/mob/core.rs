use crate::{mob::{stat::StatValue, CombatStat}, traits::Description};

pub trait IsMob {
    /// Name of a mob.
    fn prompt<'a>(&'a self) -> String;
    fn hp<'a>(&'a self) -> &'a CombatStat;
    fn take_dmg<'a>(&'a mut self, percentage: StatValue) -> bool;
    fn mp<'a>(&'a self) -> &'a CombatStat;
}

/// Core struct for mobs of all sorts.
pub struct MobCore {
    name: String,
    title: String,
    description: String,
    hp: CombatStat,
    mp: CombatStat,
}

impl Description for MobCore {
    fn description<'a>(&'a self) -> &'a str { &self.description }
    fn id<'a>(&'a self) -> &'a str { &self.name }
    fn title<'a>(&'a self) -> &'a str { &self.title }
}
