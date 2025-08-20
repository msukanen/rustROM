use serde::{Deserialize, Serialize};

use crate::item::Container;

pub enum ItemError {
    NoItemSpace(Item),
    NoContainerSpace(Container),
    TooLarge(Container),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Item;
