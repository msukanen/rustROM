use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, ClientState};

pub struct QuitCommand;

#[async_trait]
impl Command for QuitCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        ctx.player.write().await.push_state(ClientState::Logout);
    }
}
