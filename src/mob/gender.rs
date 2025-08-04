use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum Gender {
    Male,
    Female,
    Indeterminate
}
