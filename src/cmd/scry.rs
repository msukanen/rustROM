use async_trait::async_trait;
use crate::{cmd::{look::LookCommand, Command, CommandCtx}, show_help, tell_user, validate_admin};

pub struct ScryCommand;

#[async_trait]
impl Command for ScryCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_admin!(ctx);

        let who = ctx.args;
        let other = ctx.world.read().await.find_player(who);
        if let Some(found) = other {
            let you = ctx.player.clone();
                ctx.player = found.clone();
            let look = LookCommand;
            look.exec({ctx.args = ""; ctx}).await;
            ctx.player = you;
        } else {
            tell_user!(ctx.writer, "Could not locate '{}'", who);
        }
    }
}
