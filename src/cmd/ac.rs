use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, show_help_if_needed, util::Broadcast, validate_admin};

pub struct AcCommand;

#[async_trait]
impl Command for AcCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_admin!(ctx);
    }
}
