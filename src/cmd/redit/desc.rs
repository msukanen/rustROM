use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, validate_builder, ClientState};

pub struct DescCommand;

#[async_trait]
impl Command for DescCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);

        tell_user!(ctx.writer, "ROOM-EDIT :: DESC\n");
        resume_game!(ctx);
    }
}
