use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, ClientState};

pub(crate) struct SayCommand;

#[async_trait]
impl Command for SayCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if !ctx.args.is_empty() {
            let msg = format!("[{}] says: {}\n", ctx.player.name(), ctx.args);
            ctx.tx.send(msg).unwrap();
        }

        ctx.writer.write_all(ctx.prompt.as_bytes()).await.unwrap();
        ClientState::Playing(ctx.player.clone())
    }
}
