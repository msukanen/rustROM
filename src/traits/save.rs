use async_trait::async_trait;

use crate::player::save::SaveError;

#[async_trait]
pub(crate) trait DoesSave {
    async fn save(&mut self) -> Result<(), SaveError>;
}
