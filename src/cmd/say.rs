use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, mob::core::IsMob, tell_user, ClientState};

pub struct SayCommand;

#[async_trait]
impl Command for SayCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if !ctx.args.is_empty() {
            let msg = format!("[{}] says: {}\n", ctx.player.name(), ctx.args);
            ctx.tx.send(msg).unwrap();
        }

        tell_user!(ctx.writer, ctx.prompt);
        ClientState::Playing(ctx.player.clone())
    }
}
