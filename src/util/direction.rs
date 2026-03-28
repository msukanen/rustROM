//! Directions!
use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub enum DirectionError {
    CannotDeductOpposite,
}

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
    pub fn as_str(&self) -> &'static str {
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
            Self::Custom(_) => unimplemented!("DEV! Do not try 'as_str()' a Custom direction…"),
        }
    }

    pub fn opposite(&self) -> Result<Self, DirectionError> {
        Ok(match self {
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
                return Err(DirectionError::CannotDeductOpposite);
//                Self::Custom(x.clone())
            }
        })
    }

    /// Try if we get a "standard" (a.k.a. cardinal) [Direction] out of the given `value`.
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

    /// Try convert the given `value` into suitable [Direction].
    /// 
    /// Note: we accommodate for a few very common typos (and some abbreviations).
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err("Direction cannot be empty.");
        }
        Ok(Self::from(value))
    }
}

/// A trait for anything that could (even theoretically) represent cardinal [Direction].
pub trait AsDirectionCardinal {
    /// Get `self` as a cardinal [Direction], if possible.
    fn as_cardinal(&self) -> Option<Direction>;
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
    pub fn from(value: &str) -> Self {
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

    pub fn as_cardinal(&self) -> Option<Direction> {
        self.as_str().as_cardinal()
    }
}

impl AsDirectionCardinal for &str {
    fn as_cardinal(&self) -> Option<Direction> {
        match Direction::from(self) {
            Direction::Custom(_) => None,
            other => Some(other)
        }
    }
}

// A converter for convenience…
impl AsDirectionCardinal for &String {
    fn as_cardinal(&self) -> Option<Direction> { self.as_str().as_cardinal() }
}

// A converter for convenience…
impl AsDirectionCardinal for String {
    fn as_cardinal(&self) -> Option<Direction> { self.as_str().as_cardinal() }
}
