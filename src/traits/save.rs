use async_trait::async_trait;

#[derive(Debug)]
pub enum SaveError {
    Io(std::io::Error),
    JsonFormat(serde_json::Error),
    TomlFormat(toml::ser::Error),
    NoIdProvided,
}

impl From<std::io::Error> for SaveError { fn from(value: std::io::Error) -> Self { Self::Io(value)}}
impl From<serde_json::Error> for SaveError { fn from(value: serde_json::Error) -> Self { Self::JsonFormat(value)}}
impl From<toml::ser::Error> for SaveError { fn from(value: toml::ser::Error) -> Self { Self::TomlFormat(value)}}

#[async_trait]
pub trait DoesSave {
    #[must_use]
    async fn save(&mut self) -> Result<(), SaveError>;
}
