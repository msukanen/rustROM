use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, show_help_if_needed, util::Broadcast, validate_builder};

pub struct BcCommand;

#[async_trait]
impl Command for BcCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_builder!(ctx);
    }
}
