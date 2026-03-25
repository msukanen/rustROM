//! 'open' command.
//! 
//! 'open' is used — surprise much? – to open e.g. doors…

use async_trait::async_trait;

use crate::{cmd::{Command, CommandCtx}, do_in_current_room, item::inventory::Storage, show_help_if_needed, tell_user, traits::Identity, util::direction::Direction, world::exit::ExitState};

pub struct OpenCommand;

#[async_trait]
impl Command for OpenCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "open");

        do_in_current_room!(ctx, |room| {
            let mut r = room.write().await;
            let dir = Direction::from(ctx.args);
            if let Some(exit) = r.exits.get_mut(&dir) {
                match exit.state.clone() {
                    ExitState::AlwaysOpen |
                    ExitState::Open{..}   => tell_user!(ctx.writer, "It's already open…\n"),
                    ExitState::Closed{ key_id } => {
                        exit.state = ExitState::Open{ key_id };
                        tell_user!(ctx.writer, "You open the way to '{}'.\n", exit.destination);
                    },
                    ExitState::Locked { key_id } => {
                        let p = ctx.player.read().await;
                        if p.inventory.contains_bp(&key_id) {
                            exit.state = ExitState::Open{ key_id: Some(key_id.clone()) };
                            if let Some(key) = p.inventory.get(&key_id) {
                                tell_user!(ctx.writer, "You click the '{}' into the lock and the way to '{}' opens!\n", key.title(), exit.destination);
                            } else {
                                log::error!("The key '{key_id}' evaporated between .contains() and .get()?! WTF?!");
                                tell_user!(ctx.writer, "There seems to be a hole in your pocket. You can almost swear you had the right key just a moment ago…\n");
                            }
                        } else {
                            tell_user!(ctx.writer, "Unfortunately that way is locked and you don't seem to have the right key…\n");
                        }
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod cmd_open_tests {
    use std::sync::Arc;
    use tokio::{io::{AsyncBufReadExt, AsyncReadExt, BufReader, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::{broadcast, RwLock}};
    use crate::{async_client_for_tests, async_server_for_tests, item::{Item, inventory::{Container, ContainerType, Content}, key::Key}, player::Player, player_and_listener_for_tests, string::ansi::AntiAnsi, util::{Broadcast, ClientState}, world::{World, area::Area, exit::*, room::Room}, world_for_tests};
    use super::*;

    #[tokio::test]
    async fn closed_exit() {
        let _ = env_logger::try_init();
        log::info!("Preparing the stage …");
        let w = world_for_tests!();
        // close void's east exit
        {
            let mut lock = w.write().await;
            if let Some(room) = lock.rooms.get_mut("void") {
                room.write().await.set_exit_state(Direction::East, ExitState::Closed {key_id: None} );
            }
        }

        let (p, listener, addr, tx) = player_and_listener_for_tests!();
        let client_task = async_client_for_tests!(addr, "look", "goto east", "open east", "goto east");
        let server_task = async_server_for_tests!(w, listener, tx, addr, p, 4);

        // wait for the client task to finish and get the output…
        let (_, client_out) = tokio::join!(server_task, client_task);
        let output_string = client_out.unwrap();
        let output_string = output_string.strip_ansi();

        // assert that the output contains the description of BOTH rooms.
        assert!(output_string.contains("Alpha"));
        assert!(output_string.contains("the way to clearing is closed"));
        assert!(output_string.contains("Omega"));
    }

    #[tokio::test]
    async fn locked_exit() {
        let _ = env_logger::try_init();
        log::info!("Preparing the stage …");
        let w = world_for_tests!();
        // close void's east exit
        {
            let mut lock = w.write().await;
            if let Some(room) = lock.rooms.get_mut("void") {
                room.write().await.set_exit_state(Direction::East, ExitState::Locked { key_id: "abloy-key-2".into() });
            }
        }

        let (p, listener, addr, tx) = player_and_listener_for_tests!();
        let client_task = async_client_for_tests!(addr, "look", "goto east", "open east", "goto east");
        let server_task = async_server_for_tests!(w, listener, tx, addr, p, 4);

        // wait for the client task to finish and get the output…
        let (_, client_out) = tokio::join!(server_task, client_task);
        let output_string = client_out.unwrap();
        let output_string = output_string.strip_ansi();

        // assert that the output contains the description of BOTH rooms.
        assert!(output_string.contains("Alpha"));
        assert!(output_string.contains("the way to clearing is locked"));
        assert!(!output_string.contains("Omega"));
    }

    #[tokio::test]
    async fn locked_exit_with_key() {
        let _ = env_logger::try_init();
        log::info!("Preparing the stage …");
        let w = world_for_tests!();
        // close void's east exit
        {
            let mut lock = w.write().await;
            if let Some(room) = lock.rooms.get_mut("void") {
                room.write().await.set_exit_state(Direction::East, ExitState::Locked { key_id: "abloy-key-2".into() });
            }
        }

        let (p, listener, addr, tx) = player_and_listener_for_tests!();
        // give player the right key...
        {
            let mut lock = p.write().await;
            let item = Item::Key(Key::new("abloy-key-2", false));
            lock.inventory.try_insert(item).unwrap();// we trust the system… *crosses fingers*
        }
        let client_task = async_client_for_tests!(addr, "look", "goto east", "open east", "goto east");
        let server_task = async_server_for_tests!(w, listener, tx, addr, p, 4);

        // wait for the client task to finish and get the output…
        let (_, client_out) = tokio::join!(server_task, client_task);
        let output_string = client_out.unwrap();
        let output_string = output_string.strip_ansi();

        // assert that the output contains the description of BOTH rooms.
        assert!(output_string.contains("Alpha"));
        assert!(output_string.contains("opens!"));
        assert!(output_string.contains("Omega"));
    }

    #[tokio::test]
    async fn locked_exit_with_wrong_key() {
        let _ = env_logger::try_init();
        log::info!("Preparing the stage …");
        let w = world_for_tests!();
        // close void's east exit
        {
            let mut lock = w.write().await;
            if let Some(room) = lock.rooms.get_mut("void") {
                room.write().await.set_exit_state(Direction::East, ExitState::Locked { key_id: "abloy-key-2".into() });
            }
        }

        let (p, listener, addr, tx) = player_and_listener_for_tests!();
        // give player a key, but wrong one…
        {
            let mut lock = p.write().await;
            let item = Item::Key(Key::new("abloy-key-20", false));
            lock.inventory.try_insert(item).unwrap();// we trust the system… *crosses fingers*
        }
        let client_task = async_client_for_tests!(addr, "look", "goto east", "open east", "goto east");
        let server_task = async_server_for_tests!(w, listener, tx, addr, p, 4);

        // wait for the client task to finish and get the output…
        let (_, client_out) = tokio::join!(server_task, client_task);
        let output_string = client_out.unwrap();
        let output_string = output_string.strip_ansi();

        // assert that the output contains the description of BOTH rooms.
        assert!(output_string.contains("Alpha"));
        assert!(output_string.contains("right key"));
        assert!(!output_string.contains("Omega"));
    }

    #[tokio::test]
    async fn locked_exit_with_deepnest_key() {
        let _ = env_logger::try_init();
        log::info!("Preparing the stage …");
        let w = world_for_tests!();
        // close void's east exit
        {
            let mut lock = w.write().await;
            if let Some(room) = lock.rooms.get_mut("void") {
                room.write().await.set_exit_state(Direction::East, ExitState::Locked { key_id: "abloy-key-2".into() });
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
        let client_task = async_client_for_tests!(addr, "look", "goto east", "open east", "goto east");
        let server_task = async_server_for_tests!(w, listener, tx, addr, p, 4);

        // wait for the client task to finish and get the output…
        let (_, client_out) = tokio::join!(server_task, client_task);
        let output_string = client_out.unwrap();
        let output_string = output_string.strip_ansi();

        // assert that the output contains the description of BOTH rooms.
        assert!(output_string.contains("Alpha"));
        assert!(output_string.contains("opens!"));
        assert!(output_string.contains("Omega"));
    }
}
