use serde::{Deserialize, Serialize};

/// Various directions, obviously.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
pub enum Direction {
    North, East, South, West,
    NorthEast, NorthWest, SouthEast, SouthWest,
    Up, Down,
    Custom(String)
}

impl Direction {
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

    pub fn from_standard_str(value: &str) -> Result<Self, &'static str> {
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
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() { return Err("Direction cannot be empty.");}
        match value.to_lowercase().as_str() {
            "north"|"n"=> Ok(Self::North),
            "south"|"s"=> Ok(Self::South),
            "east"|"e" => Ok(Self::East),
            "west"|"w" => Ok(Self::West),
            "up"  |"u" => Ok(Self::Up),
            "down"|"d" => Ok(Self::Down),
            "northeast"|"ne" => Ok(Self::NorthEast),
            "northwest"|"nw" => Ok(Self::NorthWest),
            "southeast"|"se" => Ok(Self::SouthEast),
            "southwest"|"sw" => Ok(Self::SouthWest),
            _ => Ok(Self::Custom(value.into()))
        }
    }
}
