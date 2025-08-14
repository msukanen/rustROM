use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, sync::RwLock};
use crate::{cmd::{Command, CommandCtx}, tell_user, util::clientstate::EditorMode, validate_builder, world::room::Room, ClientState};

pub mod desc;

pub struct ReditCommand;

const NO_LORE_OR_ADMIN_ONLY: &str = "Well, unfortunately there is no recorded lore about that particular subject, as far as we know…\n";

mod player_redit_serialization {
    use std::sync::Arc;

    use serde::{Deserialize, Deserializer, Serializer};
    use tokio::sync::RwLock;

    use crate::world::room::Room;

    pub fn serialize<S: Serializer>(value: &Arc<RwLock<Room>>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        tokio::task::block_in_place(|| {
            let id = &value.blocking_read().id;
            serializer.serialize_str(&id)
        })
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<RwLock<Room>>, D::Error>
    where D: Deserializer<'de>,
    {
        let temp = String::deserialize(deserializer)?;
        let mut dummy = Room::blank();
        dummy.id = temp;
        Ok(Arc::new(RwLock::new(dummy)))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReditState {
    #[serde(with = "player_redit_serialization")]
    pub lock: Arc<RwLock<Room>>,
    pub dirty: bool,
}

#[async_trait]
impl Command for ReditCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);
        
        if match ctx.player.read().await.state() {
            ClientState::Editing { mode, .. } => match mode {
                EditorMode::Room { .. } => false,
                _ => true
            },
            _ => true
        } {
            ctx.player.write().await.push_state(ClientState::Editing { mode: EditorMode::Room });// TODO
        }
        ctx.player.read().await.state()
    }
}
