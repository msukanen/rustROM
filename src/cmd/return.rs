use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, tell_user, tell_user_unk, util::clientstate::EditorMode, ClientState};

pub struct ReturnCommand;

#[async_trait]
impl Command for ReturnCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        let old_state = ctx.player.read().await.state();
        match old_state {
            ClientState::Playing => {
                // NOTE: 'return' is a no-op when user is not within some editor context.
                tell_user_unk!(ctx.writer);
            },
            ClientState::Editing { ref mode } => match mode {
                EditorMode::Help => {
                    if ctx.player.read().await.hedit.as_ref().unwrap().dirty {
                        return tell_user!(ctx.writer, "<c red>NOTE: Unsaved edits!</c>\nUse <c yellow>save</c> first and then <c yellow>return</c> again, or if you want to discard all edits, use <c yellow>abort</c> instead of return.\n");
                    } else {
                        ctx.player.write().await.hedit = None;
                    }
                },
                EditorMode::Room => {

                }
            }
            _ => {}
        }
        ctx.player.write().await.pop_state();
    }
}
