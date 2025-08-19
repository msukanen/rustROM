use async_trait::async_trait;
use crate::{check_ro_field, cmd::{Command, CommandCtx}, show_help, tell_user, validate_admin};

pub struct SetCommand;

#[async_trait]
impl Command for SetCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_admin!(ctx);

        let parts: Vec<&str> = ctx.args.splitn(2, ' ').collect();
        if parts.len() < 2 {
            if parts[0].starts_with('?') {
                show_help!(ctx, "set");
            }
            return tell_user!(ctx.writer, "<c green>Usage:</c> set <c cyan>[FIELD] [VALUE]</c>\n       set ro <c cyan>[FIELD]</c>\n");
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
            return ;
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
    }
}
