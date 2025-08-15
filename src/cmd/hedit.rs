use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, sync::RwLock};
use crate::{cmd::{help::HelpCommand, Command, CommandCtx}, player::LoadError, resume_game, tell_user, traits::Description, util::{clientstate::EditorMode, Help}, validate_builder, world::SharedWorld, ClientState};

pub(crate) mod desc;
pub(crate) mod data;
pub(crate) mod save;
pub(crate) mod title;
pub(crate) mod alias;

pub struct HeditCommand;

mod player_hedit_serialization {
    use std::sync::Arc;

    use serde::{Deserialize, Deserializer, Serializer};
    use tokio::sync::RwLock;

    use crate::util::Help;

    pub fn serialize<S: Serializer>(value: &Arc<RwLock<Help>>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        tokio::task::block_in_place(|| {
            let id = &value.blocking_read().id;
            serializer.serialize_str(&id)
        })
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<RwLock<Help>>, D::Error>
    where D: Deserializer<'de>,
    {
        let temp = String::deserialize(deserializer)?;
        let dummy = Help::new(&temp);
        Ok(Arc::new(RwLock::new(dummy)))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HeditState {
    #[serde(with = "player_hedit_serialization")]
    pub lock: Arc<RwLock<Help>>,
    pub dirty: bool,
}

impl HeditState {
    pub async fn patch_lock(&mut self, world: &SharedWorld) -> Result<(), LoadError> {
        let id = self.lock.read().await.id.clone();
        let real = world.read().await.help.get(&id)
            .ok_or_else(|| LoadError::InvalidLockId(id))?
            .clone();
        self.lock = real;
        Ok(())
    }
}

#[async_trait]
impl Command for HeditCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);

        if ctx.args.is_empty() && ctx.player.read().await.hedit.is_none() {
            tell_user!(ctx.writer, "Which help topic you'd like to edit/create?\n");
            resume_game!(ctx);
        }

        if ctx.args.starts_with('?') {
            let cmd = HelpCommand;
            return cmd.exec({ctx.args = "hedit-internal-commands"; ctx}).await;
        }

        let mut pg = ctx.player.write().await;
        if pg.hedit.is_some() {
            if !ctx.args.is_empty() && pg.hedit.as_ref().unwrap().lock.read().await.id() != ctx.args {
                let ed = pg.hedit.as_mut().unwrap();
                if ed.dirty {
                    tell_user!(ctx.writer, "<c red>Warning!</c> Unsaved edits - '<c yellow>save</c>' or '<c yellow>abort</c>' first.\n");
                    resume_game!(ctx);
                }
            } else {
                tell_user!(ctx.writer, "Resuming edit session.\n");
                pg.push_state(ClientState::Editing { mode: EditorMode::Help });
                resume_game!(ctx);
            }
        }

        if let Some(existing_entry) = ctx.world.read().await.help.get(ctx.args) {
            pg.hedit = Some(HeditState {
                lock: existing_entry.clone(),
                dirty: false
            });
        } else {
            pg.hedit = Some(HeditState {
                lock: Arc::new(RwLock::new(Help::new(ctx.args))),
                dirty: true
            });
        }

        pg.push_state(ClientState::Editing { mode: EditorMode::Help });
        pg.state()
    }
}
