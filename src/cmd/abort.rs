use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, tell_user, tell_user_unk, util::clientstate::EditorMode, ClientState};

pub struct AbortCommand;

#[async_trait]
impl Command for AbortCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        let mut p = ctx.player.write().await;
        match p.state() {
            ClientState::Playing => {
                // NOTE: 'abort' is a no-op when user is not within some editor context.
                return tell_user_unk!(ctx.writer);
            },
            ClientState::Editing { ref mode } => {
                tell_user!(ctx.writer, "Discarding edits …\n");
                match mode {
                    EditorMode::Help => { p.hedit = None; },
                    EditorMode::Room => { p.redit = None; },
                }
            }
            _ => ()
        }

        p.pop_state();
    }
}
