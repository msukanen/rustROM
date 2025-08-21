use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use crate::{cmd::{Command, CommandCtx}};

pub struct InventoryCommand;

#[async_trait]
impl Command for InventoryCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        let contents = {
            let p = ctx.player.read().await;
            p.inventory.clone()
        };

        todo!("Todo inventory!");
        return;
    }
}
