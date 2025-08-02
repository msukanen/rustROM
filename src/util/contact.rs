use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Contact {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct AdminInfo {
    login: String,
    aliases: Option<Vec<String>>,
}
