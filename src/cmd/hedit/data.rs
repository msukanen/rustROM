use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, string::styling::RULER_LINE, tell_user, validate_builder, ClientState};

pub struct DataCommand;

#[async_trait]
impl Command for DataCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);
        let g = ctx.player.read().await;
        let entry = &g.hedit.as_ref().unwrap().entry;
        // need to show description separately so that we retain tags as-is printable w/o handling them.
        let fmt = format!(r#"
      id : {}
   <c yellow>title</c> : {}
   <c yellow>alias</c> : {:?}
   <c yellow>admin</c> : {}
 <c yellow>builder</c> : {}
    <c yellow>desc</c> : {} lines …
{}"#
        , entry.id, entry.title, entry.aliases, entry.admin, entry.builder, entry.description.lines().count(), RULER_LINE);
        tell_user!(ctx.writer, "<c green>     CMD : DATA</c>{}\n", fmt);
        let _ = ctx.writer.write_all(entry.description.as_bytes()).await;

        tell_user!(ctx.writer, "<c red>// END</c>\n");
        resume_game!(ctx);
    }
}
