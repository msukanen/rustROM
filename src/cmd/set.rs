use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{check_ro_field, cmd::{Command, CommandCtx}, resume_game, tell_command_usage, tell_user, tell_user_unk, ClientState};

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
                    format!("<c yellow>'set'</c> command is used to set/check a variety of in-game values.\n{}\n<c green>Usage:</c> set [FIELD] [VALUE]\n       set ro [FIELD]",
                            get_settables_list()));
            }
            tell_user!(ctx.writer, "<c green>Usage:</c> set [FIELD] [VALUE]\n       set ro [FIELD]\n");
            resume_game!(ctx);
        }

        let (field, value) = (parts[0], parts[1]);

        // Read-only check…
        if field.eq_ignore_ascii_case("ro") {
            match value.to_lowercase().as_str() {
                "greeting" => check_ro_field!(ctx, "greeting", greeting),
                "welcome_back" => check_ro_field!(ctx, "welcome_back", welcome_back),
                "welcome_new" => check_ro_field!(ctx, "welcome_new", welcome_new),
                _ => {tell_user!(ctx.writer, "<c red>Unknown [FIELD]</c>. See <c yellow>'set ?'</c> for a list of available options.\n");}
            }
            resume_game!(ctx);
        }

        // Write logic…
        let mut w = ctx.world.write().await;
        match field.to_lowercase().as_str() {
            "greeting" => {
                w.greeting = Some(value.to_string());
                tell_user!(ctx.writer, "Greeting updated.\n");
            },
            "welcome_back" => {
                w.welcome_back = Some(value.to_string());
                tell_user!(ctx.writer, "Welcome back message updated.\n");
            },
            "welcome_new" => {
                w.welcome_new = Some(value.to_string());
                tell_user!(ctx.writer, "Welcome new message updated.\n");
            },
            _ => {tell_user!(ctx.writer, "<c red>Unknown [FIELD]</c>. See <c yellow>'set ?'</c> for a list of available options.\n");},
        }
        resume_game!(ctx);
    }
}

fn get_settables_list() -> &'static str {
r#"Currently supported (global) sets are:

  <c blue>*</c> <c green>greeting</c>        -- the initial welcome message when someone connects.
  <c blue>*</c> <c green>welcome_back</c>    -- 'welcome back' message for returning players.
  <c blue>*</c> <c green>welcome_new</c>     -- welcome message for the new players (also alts).
"#}
