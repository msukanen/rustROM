//! Directions!
use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Various directions, obviously.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
pub enum Direction {
    North, East, South, West,
    NorthEast, NorthWest, SouthEast, SouthWest,
    Up, Down,
    Custom(String)
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Custom(c) => c,
            _ => self.as_str()
        })
    }
}

impl Direction {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Down => "down",
            Self::East => "east",
            Self::North => "north",
            Self::NorthEast => "northeast",
            Self::NorthWest => "northwest",
            Self::South => "south",
            Self::SouthEast => "southeast",
            Self::SouthWest => "southwest",
            Self::Up => "up",
            Self::West => "west",
            _ => unimplemented!("Direction::Custom cannot be &'static str, sorreh!")
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::East => Self::West,
            Self::North => Self::South,
            Self::NorthEast => Self::SouthWest,
            Self::NorthWest => Self::SouthEast,
            Self::South => Self::North,
            Self::SouthEast => Self::NorthWest,
            Self::SouthWest => Self::NorthEast,
            Self::Up => Self::Down,
            Self::West => Self::East,
            Self::Custom(x) => {
                // TODO: somehow figure out the opposite for the given 'x' ...
                log::warn!("Custom Direction::Custom({x}) - cannot deduce an opposite for…");
                Self::Custom(x.clone())
            }
        }
    }

    pub fn try_from_std(value: &str) -> Result<Self, &'static str> {
        let result = Self::try_from(value);
        match result {
            Ok(Self::Custom(_)) => Err("That is not a standard direction …"),
            _ => result // return anything else as-is.
        }
    }
}

impl TryFrom<&str> for Direction {
    type Error = &'static str;

    /// Try convert the given `value` into a suitable [Direction] enum.
    /// 
    /// Note: we accommodate for a few very common typos (and some abbreviations).
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err("Direction cannot be empty.");
        }
        Ok(Self::from(value))
    }
}

impl Direction {
    /// `From<&str>`-like `from()` impl.
    /// 
    /// # Notes
    /// Unlike [Direction::try_from], this impl does not validate empty `value`
    /// and treats it as [Direction::Custom] instead…
    //
    // Due impl conflict with actual `From<&str>` and `TryFrom<&str>`, we have
    // this `From<&str>`-like impl, which *should* see use rather rarely but
    // acts as a shortcut when `value` has already been validated by some other
    // means (e.g. [show_help_if_needed!][`crate::show_help_if_needed!`])
    // has been called before [Direction::from]).
    //
    pub(crate) fn from(value: &str) -> Self {
        let lc = value.trim().to_lowercase();
        match lc.as_str() {
            "north"|"n"|"nor"|"norht"|"nort" => Self::North,
            "south"|"s"|"sou"|"souht"|"sout" => Self::South,
            "east"|"e"|"est"|"eas" => Self::East,
            "west"|"w"|"wes" => Self::West,
            "up"  |"u" => Self::Up,
            "down"|"d"|"donw" => Self::Down,
            "northeast"|"ne" => Self::NorthEast,
            "northwest"|"nw" => Self::NorthWest,
            "southeast"|"se" => Self::SouthEast,
            "southwest"|"sw" => Self::SouthWest,
            _ => Self::Custom(lc)
        }
    }

    pub(crate) fn as_cardinal(value: &str) -> Option<Self> {
        match Direction::from(value) {
            Self::Custom(_) => None,
            other => Some(other)
        }
    }
}
