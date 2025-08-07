use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, string::styling::format_color, tell_command_usage, tell_user, tell_user_unk, ClientState};

pub struct SetCommand;

#[async_trait]
impl Command for SetCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if !ctx.player.read().await.access.is_admin() {
            tell_user_unk!(ctx.writer);
            resume_game!(ctx);
        }

        let parts: Vec<&str> = ctx.args.splitn(2, ' ').collect();
        if parts.len() < 2 {
            if parts.len() == 1 && parts[0] == "?" {
                tell_command_usage!(ctx,
                    "set",
                    "sets some (global) in-game variable",
                    r#"
<c yellow>'set'</c> command is used to set/check a variety of in-game values.
Currently supported (global) sets are:

  <c blue>*</c> <c green>greeting</c>        -- the initial welcome message when someone connects.

<c green>Usage:</c> set [FIELD] [VALUE]"#);
            }
            tell_user!(ctx.writer, format_color("<c green>Usage:</c> set <field> <value>\n"));
            resume_game!(ctx);
        }

        let (field, value) = (parts[0], parts[1]);
        if field.eq_ignore_ascii_case("ro") {
            if value.eq_ignore_ascii_case("greeting") {
                let g = {&ctx.world.read().await.greeting};
                if let Some(g) = g {
                    let desc = format!("{}\n{}\n", format_color("<c yellow>--[<c green> greeting </c>], current value:</c>"), g);
                    tell_user!(ctx.writer, desc);
                } else {
                    tell_user!(ctx.writer, "Greeting not set. Use: set greeting <new-greeting>\n");
                }
            }
            resume_game!(ctx);
        }
        let mut w = ctx.world.write().await;
        if field.eq_ignore_ascii_case("greeting") {
            w.greeting = Some(value.to_string());
            tell_user!(ctx.writer, "Greeting updated.\n");
        } else {
            tell_user!(ctx.writer, "Unknown <field>. Try 'greeting'.\n");
        }

        resume_game!(ctx);
    }
}
