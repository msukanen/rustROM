use async_trait::async_trait;
use crate::{cmd::{help::HelpCommand, Command, CommandCtx}, rerun_with_help, resume_game, string::boolean::BooleanCheckExt, tell_user, validate_builder, ClientState};

pub struct BuilderCommand;

#[async_trait]
impl Command for BuilderCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);

        if ctx.args.is_empty() {
            tell_user!(ctx.writer, "Builder-only: {}\n", ctx.player.read().await.hedit.as_ref().unwrap().entry.builder);
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
        let ed = g.hedit.as_mut().unwrap();
        ed.dirty = true;
        ed.entry.builder = ctx.args.is_true();
        tell_user!(ctx.writer, "Builder flag is now {}.\n", if ed.entry.builder {"set"} else {"unset"});

        resume_game!(ctx);
    }
}
