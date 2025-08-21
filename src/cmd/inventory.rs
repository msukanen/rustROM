use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}};

pub struct InventoryCommand;

#[async_trait]
impl Command for InventoryCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        todo!("Todo inventory!");
        return;
    }
}
