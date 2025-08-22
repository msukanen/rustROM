use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}};

pub struct PutCommand;

#[async_trait]
impl Command for PutCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        return;
    }
}
