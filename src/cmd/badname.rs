use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, validate_admin};

pub struct BadnameCommand;

#[async_trait]
impl Command for BadnameCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_admin!(ctx);
    }
}
