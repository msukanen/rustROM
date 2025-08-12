use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, resume_game, traits::Description, ClientState};

pub struct SayCommand;

#[async_trait]
impl Command for SayCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if !ctx.args.is_empty() {
            let msg = format!("[{}] says: {}\n", ctx.player.read().await.id(), ctx.args);
            ctx.tx.send(msg).unwrap();
        }
        resume_game!(ctx);
    }
}
