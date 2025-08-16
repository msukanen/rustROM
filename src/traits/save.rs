use std::fmt::Display;

use async_trait::async_trait;

#[derive(Debug)]
pub enum SaveError {
    Io(std::io::Error),
    JsonFormat(serde_json::Error),
    TomlFormat(toml::ser::Error),
    NoIdProvided,
}

impl std::error::Error for SaveError {}
impl From<std::io::Error> for SaveError { fn from(value: std::io::Error) -> Self { Self::Io(value)}}
impl From<serde_json::Error> for SaveError { fn from(value: serde_json::Error) -> Self { Self::JsonFormat(value)}}
impl From<toml::ser::Error> for SaveError { fn from(value: toml::ser::Error) -> Self { Self::TomlFormat(value)}}

impl Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::NoIdProvided => write!(f, "Cannot save entry with no ID."),
            SaveError::Io(e) => write!(f, "File I/O error: {}", e),
            SaveError::TomlFormat(e) => write!(f, "TOML ser error: {}", e),
            SaveError::JsonFormat(e) => write!(f, "JSON ser error: {}", e),
        }
    }
}

#[async_trait]
pub trait DoesSave {
    #[must_use]
    async fn save(&mut self) -> Result<(), SaveError>;
}
