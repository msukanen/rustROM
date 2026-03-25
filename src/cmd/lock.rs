//! Lock something or other…

use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;

use crate::{cmd::{Command, CommandCtx}, do_in_current_room, item::inventory::Storage, show_help_if_needed, tell_user, util::direction::Direction, world::exit::ExitState};

lazy_static! {
    static ref CLOSE_WITH_RX: Regex = Regex::new(r#"\s*(?P<what>.+)\s+with\s+(?P<with>.+)"#).unwrap();
}

pub struct LockCommand;

#[async_trait]
impl Command for LockCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "lock");

        let (what, with) = if let Some(caps) = CLOSE_WITH_RX.captures(ctx.args) {
            (caps.name("what").unwrap().as_str().trim(), Some(caps.name("with").unwrap().as_str().trim()))
        } else {
            (ctx.args, None)
        };

        do_in_current_room!(ctx, |room| {
            let mut r = room.write().await;
            let dir = Direction::from(what);
            if let Some(exit) = r.exits.get_mut(&dir) {
                match (exit.state.clone(), with) {
                    (ExitState::AlwaysOpen,..) => tell_user!(ctx.writer, "Uh, you have no clue how to close that and even less how to lock it…\n"),
                    (ExitState::Locked {..},..) => tell_user!(ctx.writer, "It's already locked…\n"),
                    (ExitState::Open {..},..) => tell_user!(ctx.writer, "No point to lock it yet. Close it first?\n"),
                    (ExitState::Closed {key_id}, with) => {
                        match (key_id, with) {
                            (Some(key_id), Some(with)) => {
                                    // does player have the right key?
                                    if !key_id.contains(&with.to_lowercase()) || !ctx.player.read().await.inventory.contains_bp(&key_id) {
                                        tell_user!(ctx.writer, "You lack the right key to lock this entrace.\n");
                                    } else {
                                        exit.state = ExitState::Locked { key_id };
                                        tell_user!(ctx.writer, "You lock the entrance to '{}'.\n", exit.destination);
                                    }
                                },
                            _ => tell_user!(ctx.writer, "You don't seem to find any locking mechanism…\n"),
                        }
                    },
                }
            } else {
                tell_user!(ctx.writer, "In theory, closing '{}' might work… if it was here, but it isn't.\n", dir);
            }
        });
    }
}
