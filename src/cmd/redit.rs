use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, util::clientstate::EditorMode, validate_builder, world::room::Room, ClientState};

pub mod desc;

pub struct ReditCommand;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReditState {
    pub entry: Room,
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
                    entry: existing_entry.read().await.clone(),
                    dirty: false
                });
            } else {
                g.redit = Some(ReditState {
                    entry: Room::blank(Some(ctx.args)),
                    dirty: true
                });
            }
        };

        g.push_state(ClientState::Editing { mode: EditorMode::Room });
        g.state()
    }
}
