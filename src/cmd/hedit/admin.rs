use async_trait::async_trait;
use crate::{cmd::{help::HelpCommand, Command, CommandCtx}, rerun_with_help, string::boolean::BooleanCheckExt, tell_user, validate_builder};

pub struct AdminCommand;

#[async_trait]
impl Command for AdminCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_builder!(ctx);

        if ctx.args.is_empty() {
            return tell_user!(ctx.writer, "Admin-only: {}\n", ctx.player.read().await.hedit.as_ref().unwrap().entry.admin);
        }

        if ctx.args.starts_with('?') {
            let cmd = HelpCommand;
            return cmd.exec({ctx.args = "hedit-admin"; ctx}).await;
        }

        if !ctx.args.is_boolean() {
            rerun_with_help!(ctx, AdminCommand);
        }

        let mut g = ctx.player.write().await;
        let ed = g.hedit.as_mut().unwrap();
        ed.dirty = true;
        ed.entry.admin = ctx.args.is_true();
        tell_user!(ctx.writer, "Admin flag is now {}.\n", if ed.entry.admin {"set"} else {"unset"});
    }
}
