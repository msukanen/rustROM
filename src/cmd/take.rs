use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, do_in_current_room, item::{inventory::Storage, weapon::WeaponType, Item}, show_help_if_needed, tell_user, traits::Identity};

pub struct TakeCommand;

#[async_trait]
impl Command for TakeCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "take");

/*         if ctx.args.is_empty() {
        do_in_current_room!(ctx, |room| {
            let item = Item::from(WeaponType::Melee);
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
        });
        }else{
 */            do_in_current_room!(ctx,|room| {
                let item = room.write().await.take_out(ctx.args);
                match item {
                    Ok(item) => {
                        let id = item.id().to_string();
                        if let Err(item) = ctx.player.write().await.inventory.try_insert(item) {
                            tell_user!(ctx.writer, "No, no can do ...\n");
                            let _ = room.write().await.try_insert(item.into());
                        } else {
                            log::debug!("Item '{}' taken from room.", id);
                            tell_user!(ctx.writer, "You nabbed {}.\n", id);
                        }
                    },
                    _=> {
                        log::debug!("No such thing as 'weapon' present...?");
                        tell_user!(ctx.writer, "Nothing taken ...\n");
                    }
                }
            });
 //       }
    }
}
