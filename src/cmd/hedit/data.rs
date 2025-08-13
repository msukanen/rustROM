use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, string::styling::RULER_LINE, tell_user, validate_builder, ClientState};

pub struct DataCommand;

#[async_trait]
impl Command for DataCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);
        let g = ctx.player.read().await;
        let g = g.hedit.clone().unwrap();
        let g = g.lock.read().await;
        // need to show description separately so that we retain tags as-is printable w/o handling them.
        let fmt = format!(r#"
      id : {}
   <c yellow>title</c> : {}
   <c yellow>alias</c> : {:?}
   <c yellow>admin</c> : {}
 <c yellow>builder</c> : {}
    <c yellow>desc</c> : {} lines …
{}"#
        , g.id, g.title, g.aliases, g.admin, g.builder, g.description.lines().count(), RULER_LINE);
        tell_user!(ctx.writer, "<c green>     CMD : DATA</c>{}\n", fmt);
        let _ = ctx.writer.write_all(g.description.as_bytes()).await;
        drop(g);
        tell_user!(ctx.writer, "<c red>// END</c>\n");
        resume_game!(ctx);
    }
}
