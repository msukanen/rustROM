//! Give something to someone else.
use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;

use crate::{cmd::{Command, CommandCtx}, do_in_current_room, force_item_to_player, item::{Item, inventory::Storage}, show_help, show_help_if_needed, tell_user, traits::{Identity, Owned}, util::{Broadcast, comm::TellFrom}};

pub struct GiveCommand;

lazy_static! {
    static ref GIVE_RX: Regex = Regex::new(r#"^\s*(?P<>)"#).unwrap();
}

#[async_trait]
impl Command for GiveCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "give");

        let args = ctx.args.split(' ').collect::<Vec<&str>>();
        if args.len() < 2 {
            show_help!(ctx, "give");
        }
        let what = args[0];
        let to = match args[1].to_lowercase().as_str() {
            "to" => if args.len() >= 3 {args[2]} else {
                show_help!(ctx, "give");
            },
            _ => args[1]
        };

        do_in_current_room!(ctx, |room| {
            let r = room.read().await;

            // See if target even is here…
            let target_id = r.players.iter().find(|(p_id,_)| p_id.contains(to)).and_then(|(p_id,_)| Some(p_id));
            if target_id.is_none() {
                tell_user!(ctx.writer, "Well, actually you see no '{}' around here…", to);
                return;
            }
            let target_id = target_id.unwrap();

            // …but no giving things to self…
            if target_id == ctx.player.read().await.id() {
                return tell_user!(ctx.writer, "Giving things to yourself? How philantrophic of you…\n");
            }

            let (giver, item) = {
                let mut p = ctx.player.write().await;
                (p.id().to_string(), p.inventory.take_out(what))
            };

            match item {
                Ok(mut item) => {
                    let w = ctx.world.read().await;
                    // see that target didn't go MIA a nanosecond ago…
                    if let Some(target_arc) = w.players.get(target_id) {
                        let mut recv = target_arc.write().await;
                        let _ = item.set_owner(target_id);
                        let item_name = item.id().to_string();
                        if let Err(e) = recv.inventory.try_insert(item) {
                            // fail: do return to sender!
                            item = Item::from(e);
                            let _ = item.set_owner(&giver);
                            force_item_to_player!(ctx, item);
                        } else {
                            tell_user!(ctx.writer, "You hand out '{}' to '{}'.\n", what, to);
                            let _ = ctx.tx.send(Broadcast::Tell {
                                subtype: None,
                                to_player: recv.id().into(),
                                message: format!("<c cyan>{}</c> gives you '{}'", giver, item_name),
                                from_player: giver.into(),
                            });
                        }
                    } else {
                        // receiver poofed - put stuff back…
                        force_item_to_player!(ctx, item);
                        tell_user!(ctx.writer, "Oh… '{}' just disappeared? Welp, happens.\n", to);
                    }
                },

                Err(_) => {
                    tell_user!(ctx.writer, "A nice gesture, theoretically, but you don't happen to have any '{}'…\n", what);
                }
            }
        });
    }
}
