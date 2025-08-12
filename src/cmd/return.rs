use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, ClientState};

pub struct ReturnCommand;

#[async_trait]
impl Command for ReturnCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        ctx.player.write().await.pop_state()
    }
}
