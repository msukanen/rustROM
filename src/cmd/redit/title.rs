//! Modify [Room] title.
use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, tell_user, validate_builder};

pub struct TitleCommand;

#[async_trait]
impl Command for TitleCommand {
    /// REdit 'title'.
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_builder!(ctx);
        
        if ctx.args.is_empty() {
            return tell_user!(ctx.writer,
                "Title/name: <c blue>'<c cyan>{}</c>'</c>.\n",
                ctx.player.read().await.redit.as_ref().unwrap().entry.title);
        }

        {
            let mut g = ctx.player.write().await;
            let ed = g.redit.as_mut().unwrap();
            ed.dirty = true;
            ed.entry.title = ctx.args.to_string();
        }

        let cmd = TitleCommand;
        cmd.exec({ctx.args = ""; ctx}).await;
    }
}
