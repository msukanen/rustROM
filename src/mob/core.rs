use serde::{Deserialize, Serialize};

use crate::{mob::{CombatStat, stat::StatValue}, traits::{Description, Identity, mob::IsMob}};

/// Core struct for mobs of all sorts.
#[derive(Debug, Deserialize, Serialize)]
pub struct MobCore {
    name: String,
    title: String,
    description: String,
    hp: CombatStat,
    mp: CombatStat,
    invis: bool,
}

impl Description for MobCore {
    fn description<'a>(&'a self) -> &'a str { &self.description }
    fn title<'a>(&'a self) -> &'a str { &self.title }
}

impl Identity for MobCore {
    fn id<'a>(&'a self) -> &'a str { &self.name }
}

impl IsMob for MobCore {
    fn hp<'a>(&'a self) -> &'a CombatStat {
        &self.hp
    }

    fn mp<'a>(&'a self) -> &'a CombatStat {
        &self.mp
    }

    async fn prompt<'a>(&'a self) -> String {
        format!("[hp ({}|{})]#> ", self.hp().current(), self.mp().current())
    }

    fn take_dmg<'a>(&'a mut self, percentage: StatValue) -> bool {
        self.hp -= percentage;
        self.hp.is_dead(true)
    }

    fn invis(&self) -> bool {
        self.invis
    }
}
