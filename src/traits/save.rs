use async_trait::async_trait;

#[derive(Debug)]
pub enum SaveError {
    Io(std::io::Error),
    Format(serde_json::Error),
}

impl From<std::io::Error> for SaveError {
    fn from(value: std::io::Error) -> Self { Self::Io(value)}
}

impl From<serde_json::Error> for SaveError {
    fn from(value: serde_json::Error) -> Self { Self::Format(value)}
}

#[async_trait]
pub trait DoesSave {
    async fn save(&self) -> Result<(), SaveError>;
}
