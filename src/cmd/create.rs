use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, do_in_current_room, item::{inventory::Storage, ItemError}, show_help_if_needed, tell_user, traits::Identity};
#[cfg(feature = "localtest")]
use crate::item::{weapon::WeaponType, Item};

pub struct CreateCommand;

#[async_trait]
impl Command for CreateCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "create");
        #[cfg(feature = "localtest")] {
            if ctx.args == "test-item" {
            do_in_current_room!(ctx, |room| {
                let mut item = Item::from(WeaponType::Melee);
                item.set_id("test-item");
                let id = item.id().to_string();
                let res = room.write().await.try_insert(item);
                match res {
                    Ok(()) => {
                        log::debug!("Item '{}' inserted to room '{}'::'{}'", id, room.read().await.id(), room.read().await.contents.id());
                    },
                    Err(e) => {
                        log::debug!("Error: {:?}", e);
                    }
                }
            });}
        }
    }
}
