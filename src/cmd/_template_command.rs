use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, ClientState};

pub struct ___Command;

#[async_trait]
impl Command for ___Command {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        return;
    }
}
