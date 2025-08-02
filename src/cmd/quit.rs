use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, ClientState};

pub(crate) struct QuitCommand;

#[async_trait]
impl Command for QuitCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        ctx.writer.write_all(b"Bye bye! See you soon again!").await.unwrap();
        ClientState::Logout(ctx.player.clone())
    }
}
