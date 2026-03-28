//! 'open' command.
//! 
//! 'open' is used — surprise much? – to open e.g. doors…

use async_trait::async_trait;

use crate::{cmd::{Command, CommandCtx}, do_in_current_room, equalize_opposite_exit_state, item::inventory::Storage, show_help_if_needed, string::rx::WHAT_WITH_ARG_RX, tell_user, traits::IdentityQuery, util::direction::Direction, world::{exit::state::{ExitStateQuery, KEY_THAT_IS_NOT_A_KEY}, room}};

pub struct OpenCommand;

#[async_trait]
impl Command for OpenCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "open");
      
        let mut try_unlock = false;
        let (what, with) = if let Some(caps) = WHAT_WITH_ARG_RX.captures(ctx.args) {
            try_unlock = true;
            (
                caps.name("what").unwrap().as_str(),
                caps.name("with").unwrap().as_str()
            )
        } else {
            (ctx.args, KEY_THAT_IS_NOT_A_KEY)
        };

        let mut equalize = false;
        let mut equalize_exit = None;
        let mut r_id = None;
        do_in_current_room!(ctx, |room| {
            let mut r = room.write().await;
            r_id = r.id().to_string().into();
            let dir = Direction::from(what);
            if let Some(exit) = r.exits.get_mut(&dir) {
                if !exit.is_closed() {
                    tell_user!(ctx.writer, "It's already open…\n");
                    return ;
                }

                if !exit.state.open() && !try_unlock {
                    tell_user!(ctx.writer, "It's not opening… Is it locked or jammed?\n");
                    return ;
                }
                
                if try_unlock && exit.key_id().contains(with) {
                    log::debug!("Unlock attempt with '{with}…'");
                    // find the correct key?
                    if let Some(key) = ctx.player.read().await.inventory.specs_of(exit.key_id()) {
                        exit.state.force_unlock();
                        exit.state.open();
                        tell_user!(ctx.writer, "You click the '{}' into the lock and the way to '{}' opens!\n", key.title(), exit.destination);
                    } else {
                        tell_user!(ctx.writer, "Unfortunately that way is locked and you don't seem to have the right key…\n");
                    }
                } else {
                    // find the specified but still wrong key?
                    if let Some(key) = ctx.player.read().await.inventory.specs_of(with) {
                        tell_user!(ctx.writer, "No matter how you try, '{}' doesn't fit…\n", key.title());
                    } else {
                        tell_user!(ctx.writer, "Unfortunately that way is locked and you don't seem to have the right key…\n");
                    }
                }

                equalize = true;
                equalize_exit = exit.clone().into();
            }
        });

        equalize_opposite_exit_state!(equalize, ctx, r_id, equalize_exit);
    }
}

#[cfg(test)]
mod cmd_open {
    use std::sync::Arc;
    use tokio::{io::{AsyncBufReadExt, AsyncReadExt, BufReader, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::{broadcast, RwLock}};
    use crate::{async_client_for_tests, async_server_for_tests, item::{Item, inventory::{Container, ContainerType, Content}, key::Key}, player::Player, player_and_listener_for_tests, string::ansi::AntiAnsi, util::{Broadcast, ClientState}, world::{World, area::Area, exit::{state::ExitState, *}, room::Room}, world_for_tests};
    use super::*;

    /// Open a closed entry.
    #[tokio::test]
    async fn closed() {
        let _ = env_logger::try_init();
        log::info!("Preparing the stage …");
        let w = world_for_tests!();
        // close void's east exit
        {
            let mut lock = w.write().await;
            if let Some(room) = lock.rooms.get_mut("void") {
                room.write().await.set_exit_state(Direction::East, ExitState::Closed {key_id: None, jam: None} );
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

    /// Opening attempt with no key specified (nor any in possession).
    #[tokio::test]
    async fn locked_no_key() {
        let _ = env_logger::try_init();
        log::info!("Preparing the stage …");
        let w = world_for_tests!();
        // close void's east exit
        {
            let mut lock = w.write().await;
            if let Some(room) = lock.rooms.get_mut("void") {
                room.write().await.set_exit_state(Direction::East, ExitState::Locked { key_id: "abloy-key-2".into(), jam: None });
            }
        }

        let (p, listener, addr, tx) = player_and_listener_for_tests!();
        let client_task = async_client_for_tests!(addr, "look", "goto east", "open east");
        let server_task = async_server_for_tests!(w, listener, tx, addr, p, 3);

        // wait for the client task to finish and get the output…
        let (_, client_out) = tokio::join!(server_task, client_task);
        let output_string = client_out.unwrap();
        let output_string = output_string.strip_ansi();

        // assert that the output contains the description of BOTH rooms.
        assert!(output_string.contains("Alpha"));
        assert!(output_string.contains("jammed"));
    }

    /// Opening attempt with correct key.
    #[tokio::test]
    async fn locked_right_key() {
        let _ = env_logger::try_init();
        log::info!("Preparing the stage …");
        let w = world_for_tests!();
        // close void's east exit
        {
            let mut lock = w.write().await;
            if let Some(room) = lock.rooms.get_mut("void") {
                room.write().await.set_exit_state(Direction::East, ExitState::Locked { key_id: "abloy-key-2".into(), jam: None });
            }
        }

        let (p, listener, addr, tx) = player_and_listener_for_tests!();
        // give player the right key...
        {
            let mut lock = p.write().await;
            let item = Item::Key(Key::new("abloy-key-2", false));
            lock.inventory.try_insert(item).unwrap();// we trust the system… *crosses fingers*
        }
        let client_task = async_client_for_tests!(addr, "look", "goto east", "open east", "open east with abloy", "goto east");
        let server_task = async_server_for_tests!(w, listener, tx, addr, p, 5);

        // wait for the client task to finish and get the output…
        let (_, client_out) = tokio::join!(server_task, client_task);
        let output_string = client_out.unwrap();
        let output_string = output_string.strip_ansi();

        // assert that we end up at the destination…
        assert!(output_string.contains("Omega"));
    }

    /// Opening attempt with wrong key.
    #[tokio::test]
    async fn locked_wrong_key_attempt() {
        let _ = env_logger::try_init();
        log::info!("Preparing the stage …");
        let w = world_for_tests!();
        // close void's east exit
        {
            let mut lock = w.write().await;
            if let Some(room) = lock.rooms.get_mut("void") {
                room.write().await.set_exit_state(Direction::East, ExitState::Locked { key_id: "abloy-key-2".into(), jam: None });
            }
        }

        let (p, listener, addr, tx) = player_and_listener_for_tests!();
        // give player a key, but wrong one…
        {
            let mut lock = p.write().await;
            let item = Item::Key(Key::new("abloy-key-20", false));
            lock.inventory.try_insert(item).unwrap();// we trust the system… *crosses fingers*
        }
        let client_task = async_client_for_tests!(addr, "look", "goto east", "open east", "open east with abloy", "goto east");
        let server_task = async_server_for_tests!(w, listener, tx, addr, p, 5);

        // wait for the client task to finish and get the output…
        let (_, client_out) = tokio::join!(server_task, client_task);
        let output_string = client_out.unwrap();
        let output_string = output_string.strip_ansi();

        // assert that the output contains the title of both rooms along
        // - initial failure
        // - success with usage of right key
        assert!(output_string.contains("Alpha"));
        assert!(output_string.contains("jammed?"));
        assert!(output_string.contains("locked"));
        assert!(!output_string.contains("Omega"));
    }
}
