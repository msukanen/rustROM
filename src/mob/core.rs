use crate::mob::{stat::StatValue, CombatStat};

pub trait IsMob {
    /// Name of a mob.
    fn name<'a>(&'a self) -> &'a str;
    fn prompt<'a>(&'a self) -> String;
    fn hp<'a>(&'a self) -> &'a CombatStat;
    fn take_dmg<'a>(&'a mut self, percentage: StatValue) -> bool;
    fn mp<'a>(&'a self) -> &'a CombatStat;
}

/// Core struct for mobs of all sorts.
pub struct MobCore {
    hp: CombatStat,
    mp: CombatStat,
}
