//! Close something or other…
//! 
//! 'close' closes a door or such.

use async_trait::async_trait;
use regex::Regex;

use crate::{cmd::{Command, CommandCtx}, do_in_current_room, item::inventory::Storage, show_help_if_needed, tell_user, traits::Identity, util::direction::Direction, world::exit::ExitState};

pub struct CloseCommand;

#[async_trait]
impl Command for CloseCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "close");

        let close_with_rx = Regex::new(r#"\s*(?P<what>.+)\s+with\s+(?P<with>.+)"#).unwrap();
        let (what, with) = if let Some(caps) = close_with_rx.captures(ctx.args) {
            (caps.name("what").unwrap().as_str().trim(), Some(caps.name("with").unwrap().as_str().trim()))
        } else {
            (ctx.args, None)
        };

        do_in_current_room!(ctx, |room| {
            let mut r = room.write().await;
            let dir = Direction::from(what);
            let r_id = r.id().to_string();
            if let Some(exit) = r.exits.get_mut(&dir) {
                let mut state_changed = false;
                match (exit.state.clone(), with) {
                    (ExitState::AlwaysOpen,..) => tell_user!(ctx.writer, "Well, honestly — you have no faintest clue how to close that…\n"),
                    (ExitState::Locked {..},..) => tell_user!(ctx.writer, "It's already closed and locked…\n"),
                    
                    (ExitState::Open {key_id}, with)   |
                    (ExitState::Closed {key_id}, with) => {
                        match (key_id, with) {
                            (Some(key_id), Some(with)) => {
                                    // does player have the right key?
                                    if !key_id.contains(&with.to_lowercase()) || !ctx.player.read().await.inventory.contains_bp(&key_id) {
                                        state_changed = exit.state.close();
                                        tell_user!(ctx.writer, "You lack the right key to lock this entrace, but you close it nonetheless.\n");
                                    } else {
                                        state_changed = exit.state.lock_with(&key_id).is_ok_and(|v|v);
                                        log::debug!("'close with' locked {:?}", exit);
                                        tell_user!(ctx.writer, "You close and lock the entrance to '{}'.\n", exit.destination);
                                    }
                                },
                            // close but no auto-lock…
                            _ => if matches!(exit.state, ExitState::Closed {..}) {
                                    tell_user!(ctx.writer, "It's already closed…\n")
                                } else {
                                    state_changed = exit.state.close();
                                    tell_user!(ctx.writer, "You close the way to '{}'.\n", exit.destination);
                                },

                        }
                    },
                }

                    
                // close/lock the other side, if possible/needed:
                if state_changed {
                    let mut wlock = ctx.world.write().await;
                    if let Some(oroom) = wlock.rooms.get_mut(&exit.destination) {
                        let mut orlock = oroom.write().await;
                        for oexit in orlock.exits.values_mut() {
                            if oexit.state == ExitState::AlwaysOpen {
                                continue;
                            }

                            if oexit.destination == r_id {
                                oexit.state = exit.state.clone()
                            }
                        }
                    }
                }
            } else {
                tell_user!(ctx.writer, "In theory, closing '{}' might work… if it was here, but it isn't.\n", dir);
            }
        });
    }
}
