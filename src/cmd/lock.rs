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
                                        log::debug!("Locked: '{:?}'", exit);
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

#[cfg(test)]
mod cmd_lock_tests {
    use super::*;
    use std::sync::Arc;
    use tokio::{io::{AsyncBufReadExt, AsyncReadExt, BufReader, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::{broadcast, RwLock}};
    use crate::{async_client_for_tests, async_server_for_tests, item::{Item, inventory::*, key::Key}, player::Player, player_and_listener_for_tests, string::ansi::AntiAnsi, util::{Broadcast, ClientState}, world::{World, area::Area, exit::*, room::Room}, world_for_tests};

    #[tokio::test]
    async fn lock_open_exit() {
        let _ = env_logger::try_init();
        log::info!("Preparing the stage …");
        let w = world_for_tests!();
        // assign key id for "void"'s lock
        {
            let mut lock = w.write().await;
            if let Some(room) = lock.rooms.get_mut("void") {
                room.write().await.set_exit_state(Direction::East, ExitState::Open { key_id: Some("abloy-key-2".into()) });
            }
        }

        let (p, listener, addr, tx) = player_and_listener_for_tests!();
        // give player the right key...
        {
            let item = Item::Key(Key::new("abloy-key-2", false));
            let mut bag = Item::Container(Container::Backpack(Content::from(ContainerType::Backpack)));
            bag.try_insert(item).unwrap(); // we trust the system… *crosses fingers*

            let mut lock = p.write().await;
            lock.inventory.try_insert(bag).unwrap();// we trust the system… *crosses fingers*
        }
        let client_task = async_client_for_tests!(addr,
            "look",
            "lock east",
            "close east with abloy",
            "goto east",
            "open east",
            "goto east"
        );
        let server_task = async_server_for_tests!(w, listener, tx, addr, p, 6);

        // wait for the client task to finish and get the output…
        let (_, client_out) = tokio::join!(server_task, client_task);
        let output_string = client_out.unwrap();
        let output_string = output_string.strip_ansi();

        // assert that the output contains the description of BOTH rooms.
        assert!(output_string.contains("Alpha"));
        assert!(output_string.contains("Omega"));
    }
}
