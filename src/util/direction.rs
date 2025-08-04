use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Direction {
    North, East, South, West,
    NorthEast, NorthWest, SouthEast, SouthWest,
    Up, Down,
    Custom(String)
}
