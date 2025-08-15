use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{help::HelpCommand, Command, CommandCtx}, resume_game, string::boolean::BooleanCheckExt, tell_user, validate_builder, ClientState};

pub struct AdminCommand;

#[async_trait]
impl Command for AdminCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);

        if ctx.args.is_empty() {
            let g = ctx.player.read().await;
            let g = g.hedit.as_ref().unwrap().lock.read().await;
            tell_user!(ctx.writer, "Admin-only: {}\n", g.admin);
            resume_game!(ctx);
        }

        if ctx.args.starts_with('?') {
            let cmd = HelpCommand;
            return cmd.exec({ctx.args = "hedit-admin"; ctx}).await;
        }

        let mut g = ctx.player.write().await;
        let g = g.hedit.as_mut().unwrap();
        g.dirty = true;
        let mut g = g.lock.write().await;
        g.admin = ctx.args.is_true();

        resume_game!(ctx);
    }
}
