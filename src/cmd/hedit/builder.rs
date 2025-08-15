use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{help::HelpCommand, Command, CommandCtx}, rerun_with_help, resume_game, string::boolean::BooleanCheckExt, tell_user, validate_builder, ClientState};

pub struct BuilderCommand;

#[async_trait]
impl Command for BuilderCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);

        if ctx.args.is_empty() {
            let g = ctx.player.read().await;
            let g = g.hedit.as_ref().unwrap().lock.read().await;
            tell_user!(ctx.writer, "Builder-only: {}\n", g.builder);
            resume_game!(ctx);
        }

        if ctx.args.starts_with('?') {
            let cmd = HelpCommand;
            return cmd.exec({ctx.args = "hedit-builder"; ctx}).await;
        }

        if !ctx.args.is_boolean() {
            rerun_with_help!(ctx, BuilderCommand);
        }

        let mut g = ctx.player.write().await;
        let g = g.hedit.as_mut().unwrap();
        g.dirty = true;
        let mut g = g.lock.write().await;
        g.builder = ctx.args.is_true();
        tell_user!(ctx.writer, "Builder flag is now {}.\n", if g.builder {"set"} else {"unset"});

        resume_game!(ctx);
    }
}
