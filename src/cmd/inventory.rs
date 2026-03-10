use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, item::inventory::{Storage, StorageCapacity}, tell_user};

pub struct InventoryCommand;

#[async_trait]
impl Command for InventoryCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        let p = ctx.player.read().await;
        let inv = &p.inventory;

        if inv.is_empty() {
            tell_user!(ctx.writer, "Your inventory seems to be <c brown>empty</c>…\n");
            return ;
        }

        let mut output = format!(
            "<c yellow>Inventory</c> (Slots: {}/{} | Total Objects: {})\n",
            inv.items().len(), inv.capacity(), inv.num_items()
        );
        
        for (id, item) in inv.items() {
            let extra = if item.num_items() > 0 {
                format!(" [contains {} items]", item.num_items())
            } else {"".into()};

            output.push_str(&format!("  - <c cyan>{id}</c>{extra}\n",));
        }

        tell_user!(ctx.writer, "{}", output);
    }
}
