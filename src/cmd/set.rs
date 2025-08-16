use async_trait::async_trait;
use crate::{check_ro_field, cmd::{help::HelpCommand, Command, CommandCtx}, resume_game, tell_user, validate_admin, ClientState};

pub struct SetCommand;

#[async_trait]
impl Command for SetCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_admin!(ctx);

        let parts: Vec<&str> = ctx.args.splitn(2, ' ').collect();
        if parts.len() < 2 {
            if parts[0].starts_with('?') {
                ctx.args = "set";
                let help = HelpCommand;
                return help.exec(ctx).await;
            }
            tell_user!(ctx.writer, "<c green>Usage:</c> set <c cyan>[FIELD] [VALUE]</c>\n       set ro <c cyan>[FIELD]</c>\n");
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
