use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, do_in_current_room, item::{inventory::Storage, Item, ItemError}, show_help_if_needed, tell_user, traits::{owned::OwnerError, Identity, Owned}};
#[cfg(feature = "localtest")]
use crate::item::{weapon::WeaponType, Item};

pub struct TakeCommand;

#[async_trait]
impl Command for TakeCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "take");
        do_in_current_room!(ctx,|room| {
            let item = room.write().await.take_out(ctx.args);
            match item {
                Ok(mut item) => {
                    let id = item.id().to_string();

                    if !item.is_owned() {
                        let p = ctx.player.read().await;
                        let p_id = p.id();
                        let err_o = item.set_owner(p_id);
                        let err_oo = item.set_original_owner(p_id);
                        log::debug!("Item '{}' is now owned by '{}': EO {err_o:?} / EOO {err_oo:?}", id, p_id);
                    }

                    if let Err(item) = ctx.player.write().await.inventory.try_insert(item) {
                        // tell the user why exactly taking the item didn't work...
                        match &item {
                            ItemError::NoSpace(_) => {tell_user!(ctx.writer, "Uh oh - can't carry that much. Make some space in your inventory first…");},
                            ItemError::TooLarge(_) => {tell_user!(ctx.writer, "Yeah, but no - it's too large to pick up.");},
                            _ => unimplemented!("should not happen")// should not happen…
                        }
                        // put the item back into the room so that it doesn't just vanish forever.
                        if let Err(e) = room.write().await.try_insert(item.into()) {
                            // and if THAT fails... stash to lost_and_found
                            ctx.world.write().await.lost_and_found.insert(e.id().to_string(), e);
                        }
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
    }
}
