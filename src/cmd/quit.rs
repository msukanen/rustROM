use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, ClientState};

pub(crate) struct QuitCommand;

#[async_trait]
impl Command for QuitCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        ClientState::Logout(ctx.player.clone())
    }
}
