use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Contact {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AdminInfo {
    login: String,
    aliases: Option<Vec<String>>,
}
