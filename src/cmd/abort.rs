use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, tell_user_unk, util::clientstate::EditorMode, ClientState};

pub struct AbortCommand;

#[async_trait]
impl Command for AbortCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        let old_state = ctx.player.read().await.state();
        match old_state {
            ClientState::Playing => {
                // NOTE: 'abort' is a no-op when user is not within some editor context.
                tell_user_unk!(ctx.writer);
            },
            ClientState::Editing { ref mode } => match mode {
                EditorMode::Help => {
                    tell_user!(ctx.writer, "Discarding edits …\n");
                    ctx.player.write().await.hedit = None;
                },
                EditorMode::Room => {

                }
            }
            _ => {}
        }
        ctx.player.write().await.pop_state()
    }
}
