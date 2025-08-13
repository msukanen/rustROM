use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, string::styling::RULER_LINE, tell_user, validate_builder, ClientState};

pub struct DescCommand;

#[async_trait]
impl Command for DescCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);
        tell_user!(ctx.writer,
            "{}\n{}<c red>// END</c>\n",
            RULER_LINE,
            ctx.player.read().await.hedit.as_ref().unwrap().lock.read().await.description
        );
        resume_game!(ctx);
    }
}
