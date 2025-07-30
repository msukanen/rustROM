use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub(crate) enum Gender {
    Male,
    Female,
    Indeterminate
}
