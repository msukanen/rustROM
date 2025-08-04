use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, tell_user_unk, ClientState};

pub struct SetCommand;

#[async_trait]
impl Command for SetCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if !ctx.player.access.is_admin() {
            tell_user_unk!(ctx.writer, ctx.prompt);
            resume_game!(ctx);
        }

        let parts: Vec<&str> = ctx.args.splitn(2, ' ').collect();
        if parts.len() < 2 {
            if parts.len() == 1 && parts[0] == "?" {
                tell_user!(ctx.writer, "\
'set' command is used to set/check a variety of in-game values.  Currently\n\
supported (global) sets are:\n\
\n\
  - greeting        -- the initial welcome message when someone connects.\n\
\n");
            }
            tell_user!(ctx.writer, "Usage: set <field> <value>\n{}", ctx.prompt);
            resume_game!(ctx);
        }

        let (field, value) = (parts[0], parts[1]);
        if field.eq_ignore_ascii_case("ro") {
            if value.eq_ignore_ascii_case("greeting") {
                let g = {&ctx.world.read().await.greeting};
                if let Some(g) = g {
                    tell_user!(ctx.writer, "---greeting, current value:---\n{}\n", g);
                } else {
                    tell_user!(ctx.writer, "Greeting not set. Use: set greeting <new-greeting>\n");
                }
            }
            tell_user!(ctx.writer, "{}", ctx.prompt);
            resume_game!(ctx);
        }
        let mut w = ctx.world.write().await;
        if field.eq_ignore_ascii_case("greeting") {
            w.greeting = Some(value.to_string());
            tell_user!(ctx.writer, "Greeting updated.\n");
        } else {
            tell_user!(ctx.writer, "Unknown <field>. Try 'greeting'.\n");
        }

        tell_user!(ctx.writer, "{}", ctx.prompt);
        resume_game!(ctx);
    }
}
