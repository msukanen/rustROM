use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, validate_admin};

pub struct TstCommand;

#[async_trait]
impl Command for TstCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        // NOTE: ensure that template/tst command is always admin-only and local!
        validate_admin!(ctx);
    }
}
