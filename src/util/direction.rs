use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Direction {
    North, East, South, West,
    NorthEast, NorthWest, SouthEast, SouthWest,
    Up, Down,
    Custom(String)
}

impl TryFrom<&str> for Direction {
    type Error = &'static str;
        
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "north"|
            "n" => Ok(Self::North),
            "south"|
            "s" => Ok(Self::South),
            "east"|
            "e" => Ok(Self::East),
            "west"|
            "w" => Ok(Self::West),
            "northeast"|
            "ne" => Ok(Self::NorthEast),
            "northwest"|
            "nw" => Ok(Self::NorthWest),
            "southeast"|
            "se" => Ok(Self::SouthEast),
            "southwest"|
            "sw" => Ok(Self::SouthWest),
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            _ => Err("Invalid direction. No such direction specified…")
        }
    }
}
