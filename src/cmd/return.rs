use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, tell_user, tell_user_unk, ClientState};

pub struct ReturnCommand;

#[async_trait]
impl Command for ReturnCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        let old_state = ctx.player.read().await.state();
        match old_state {
            ClientState::Playing => {tell_user_unk!(ctx.writer);},
            _ => {}
        }
        ctx.player.write().await.pop_state()
    }
}
