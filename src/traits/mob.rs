use crate::mob::{CombatStat, stat::StatValue};

pub trait IsMob {
    /// Name of a mob.
    async fn prompt<'a>(&'a self) -> String;
    
    /// Mob's hit points (hp).
    fn hp<'a>(&'a self) -> &'a CombatStat;
    /// "Take this!"…
    /// 
    /// # Returns
    /// `true` if still kicking.
    fn take_dmg<'a>(&'a mut self, percentage: StatValue) -> bool;
    
    /// Mob's magic points/potential (mp).
    fn mp<'a>(&'a self) -> &'a CombatStat;

    /// Check if mob is currently invisible.
    fn invis(&self) -> bool;
}
