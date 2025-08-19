use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, string::styling::RULER_LINE, tell_user, validate_builder};

pub struct DataCommand;

#[async_trait]
impl Command for DataCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_builder!(ctx);
        let g = ctx.player.read().await;
        let entry = &g.hedit.as_ref().unwrap().entry;
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
        // need to show description separately, in raw form, so that we retain tags as-is w/o handling them.
        let _ = ctx.writer.write_all(entry.description.as_bytes()).await;
        tell_user!(ctx.writer, "<c red>// END</c>\n");
    }
}
