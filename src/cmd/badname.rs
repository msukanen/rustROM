use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, resume_game, validate_admin, ClientState};

pub struct BadnameCommand;

#[async_trait]
impl Command for BadnameCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_admin!(ctx);

        resume_game!(ctx);
    }
}
