//! Mob stats.
use std::{cmp::Ordering, ops::{AddAssign, SubAssign}};

use async_trait::async_trait;
//
use serde::{Deserialize, Serialize};

use crate::traits::tickable::Tickable;
/// Stat-type definitions for default() generation.
pub enum StatType {
    /// Hit Points.
    HP,
    /// Mental Points.
    MP,
}

/// A 'value type' for stat internals.
pub type StatValue = f32;

/// Combat stats.
/// 
/// FYI: all combat stats are percentage based values.
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum CombatStat {
    /// Hit Points. If `current` falls below zero, you are/will be likely dead/dying…
    HP { current: StatValue, max: StatValue },
    /// Mental Points. Measure of mental strain, etc.
    MP {
        /// If `current` falls to zero or less, you are/will be likely unconscious.
        current: StatValue,   // current state
        max: StatValue,       // max. when fresh
        /// Drain per tick (due maintained effort of some sort, etc.).
        drain: StatValue,
    },
}

impl CombatStat {
    pub fn default( stat_type: StatType ) -> Self {
        match stat_type {
            StatType::HP => Self::HP { current: 100.0 as StatValue, max: 100.0 as StatValue },
            StatType::MP => Self::MP { current: 100.0 as StatValue, max: 100.0 as StatValue, drain: 0.0 }
        }
    }

    pub fn current(&self) -> StatValue {
        match self {
            Self::HP { current, .. } |
            Self::MP { current, .. } => *current,
        }
    }

    pub fn set_current(&mut self, new: StatValue) {
        match self {
            Self::HP { current, .. } |
            Self::MP { current, .. } => *current = new,
        }
    }

    pub fn max(&self) -> StatValue {
        match self {
            Self::HP { max, .. } |
            Self::MP { max, .. } => *max,
        }
    }

    pub fn reset(&mut self) {
        match self {
            Self::HP { current, max } |
            Self::MP { current, max, .. } => *current = *max,
        }
    }

    pub fn is_unconscious(&self) -> bool {
        match self {
            Self::HP { current, .. } |
            Self::MP { current, .. } => *current <= 0.0001,
        }
    }

    pub fn is_dead(&self, is_mob: bool) -> bool {
        match self {
            Self::HP { current, .. } => *current < if is_mob {0.0} else {-10.0},
            _ => false
        }
    }
}

#[async_trait]
impl Tickable for CombatStat {
    async fn tick(&mut self, _: u64) {
        match self {
            Self::HP { .. } => {},
            Self::MP { current, drain, .. } => *current -= *drain,
        }
    }
}

impl AddAssign<StatValue> for CombatStat {
    fn add_assign(&mut self, rhs: StatValue) {
        match self {
            Self::HP { current, .. } |
            Self::MP { current, .. } => *current += rhs,
        }
    }
}

impl SubAssign<StatValue> for CombatStat {
    fn sub_assign(&mut self, rhs: StatValue) {
        match self {
            Self::HP { current, .. } |
            Self::MP { current, .. } => *current -= rhs,
        }
    }
}

impl PartialEq<StatValue> for CombatStat {
    fn eq(&self, other: &StatValue) -> bool {
        (self.current() - other).abs() < 0.001
    }
}

impl PartialOrd<StatValue> for CombatStat {
    fn partial_cmp(&self, other: &StatValue) -> Option<Ordering> {
        Some(if self == other {
            Ordering::Equal
        } else {
            self.current().partial_cmp(other)
                .expect("No nuns allowed. Just monks. Someone NaN'd a float. Go punish.")
        })
    }
}
