use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, sync::RwLock};
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, traits::Description, util::clientstate::EditorMode, validate_builder, world::room::Room, ClientState};

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

        if ctx.args.is_empty() && ctx.player.read().await.redit.is_none() {
            tell_user!(ctx.writer,
                "ROOM-ID missing and no previous REdit session stored.\n\
                Which room you want to edit? In case of the current one, use '<c yellow>redit this</c>'\n");
            resume_game!(ctx);
        }

        let mut g = ctx.player.write().await;
        if g.redit.is_none() {
            if let Some(existing_entry) = ctx.world.read().await.rooms.get(ctx.args) {
                g.redit = Some(ReditState {
                    lock: existing_entry.clone(),
                    dirty: false
                });
            } else {
                g.redit = Some(ReditState {
                    lock: Arc::new(RwLock::new({
                        let mut room = Room::blank();
                        room.id = ctx.args.into();
                        room
                    })),
                    dirty: true
                });
            }
        };

        g.push_state(ClientState::Editing { mode: EditorMode::Room });
        g.state()
    }
}
