use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, tell_user, validate_builder};

pub struct DescCommand;

#[async_trait]
impl Command for DescCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_builder!(ctx);

        tell_user!(ctx.writer, "ROOM-EDIT :: DESC\n");
    }
}
