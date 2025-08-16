use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, validate_builder, ClientState};

pub struct TitleCommand;

#[async_trait]
impl Command for TitleCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);
        if ctx.args.is_empty() {
            tell_user!(ctx.writer,
                "Title/name: <c blue>'<c cyan>{}</c>'</c>.\n",
                ctx.player.read().await.hedit.as_ref().unwrap().entry.title);
            resume_game!(ctx);
        }

        {
            let mut g = ctx.player.write().await;
            let ed = g.hedit.as_mut().unwrap();
            ed.dirty = true;
            ed.entry.title = ctx.args.to_string();
            drop(g);
        }

        let cmd = TitleCommand;
        cmd.exec({ctx.args = ""; ctx}).await
    }
}
