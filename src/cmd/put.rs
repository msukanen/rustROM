use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, cmd_exec, do_in_current_room, force_item_to_player, item::{inventory::Storage, Item, ItemError}, show_help, tell_user, traits::Identity};

pub struct PutCommand;

#[async_trait]
impl Command for PutCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        let args = ctx.args.splitn(2, ' ').collect::<Vec<&str>>();
        if args.len() < 2 || args[0].starts_with('?') {
            show_help!(ctx, "put");
        }
        let (what, where_to) = (args[0], args[1]);
        let mut p = ctx.player.write().await;
        if let Ok(item) = p.inventory.take_out(what) {
            drop(p);// prevent deadlocks
            match where_to {
                "ground" => {
                    let r = ctx.player.read().await.room.upgrade().unwrap();
                    let mut lock = r.write().await;
                    if let Err(e) = lock.contents.try_insert(item) {
                        drop(lock);// prevent deadlocks
                        drop(r);// prevent deadlocks
                        let item = Item::from(e);
                        force_item_to_player!(ctx, item);
                    }
                },
                _ => {
                    let r = ctx.player.read().await.room.upgrade().unwrap();
                    let mut lock = r.write().await;
                    if let Ok(where_to) = lock.contents.contains_r(where_to) {
                        let Some(c) = lock.contents.items_mut().get_mut(&where_to) else {panic!("OMG! Where did '{where_to}' go?! contains_r() DID find it...")};
                        let i_id = item.id().to_string();
                        if let Err(e) = c.try_insert(item) {
                            drop(lock);
                            match &e {
                                ItemError::NoSpace(_) => tell_user!(ctx.writer, "Not enough space in '{}' for '{}.\n'", where_to, e.id()),
                                ItemError::NotContainer(_) => tell_user!(ctx.writer, "'{}' isn't a container, unfortunately…\n", e.id()),
                                ItemError::TooLarge(_) => tell_user!(ctx.writer, "No way that '{}' would fit into '{}' without some serious shrinking!\n", e.id(), where_to),
                                ItemError::NotFound => {
                                    tell_user!(ctx.writer, "I'm not quite sure what happened… '{}' was here, but now it isn't! Uh oh, the world is spinning…", i_id);
                                    panic!("Item '{}' vanished in transit?! Brace for impact!", i_id);
                                }
                            }
                            let item = Item::from(e);
                            force_item_to_player!(ctx, item);
                        }
                    } else {
                        tell_user!(ctx.writer, "No matter how hard to look, you don't find anything '{}' related around…", where_to);
                    }
                }
            };
        } else {
            tell_user!(ctx.writer, "No matter how hard you search, you find no '{}' in your possession…\n", what);
        }
    }
}
