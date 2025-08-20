use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, show_help, tell_user, validate_builder};

pub struct DescCommand;

#[async_trait]
impl Command for DescCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_builder!(ctx);
        if ctx.args.starts_with('?') {
            show_help!(ctx, "edit-desc");
        }

        let desc = ctx.world.read().await.rooms.get(&ctx.player.read().await.location).unwrap().clone();
        tell_user!(ctx.writer, "ROOM-EDIT :: DESC\n{}\n", desc.read().await.description);
    }
}
