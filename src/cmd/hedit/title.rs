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
                ctx.player.read().await.hedit.as_ref().unwrap().lock.read().await.title);
            resume_game!(ctx);
        }

        {
            let mut g = ctx.player.write().await;
            let g = g.hedit.as_mut().unwrap();
            g.dirty = true;
            let mut g = g.lock.write().await;
            g.title = ctx.args.to_string();
            drop(g);
        }
        let cmd = TitleCommand;
        ctx.args = "";
        cmd.exec(ctx).await
    }
}
