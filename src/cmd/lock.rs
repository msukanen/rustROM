//! Lock something or other…

use async_trait::async_trait;

use crate::{cmd::{Command, CommandCtx}, do_in_current_room, equalize_opposite_exit_state, show_help_if_needed, string::rx::WHAT_WITH_ARG_RX, tell_user, traits::Identity, util::direction::Direction, world::exit::{KeyError, jam::JamState, state::{ExitState, KEY_THAT_IS_NOT_A_KEY}}};

pub struct LockCommand;

#[async_trait]
impl Command for LockCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "lock");

        let (what, with) = if let Some(caps) = WHAT_WITH_ARG_RX.captures(ctx.args) {
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
                match exit.state.lock_with(with) {
                    Ok(true) => {
                            equalize = true;
                            equalize_exit = exit.clone().into();
                            tell_user!(ctx.writer, "You lock the entrance to '{}'.\n", exit.destination)
                        },
                    Ok(_) => if matches!(exit.state, ExitState::Closed { jam: Some(JamState::WholeExit(_)),.. }) {
                            tell_user!(ctx.writer, "Uhm, well – the door is stuck, but so is the lock… No can do.\n");
                        } else {
                            tell_user!(ctx.writer, "It's already locked…\n");
                        },
                    Err(e) => match e {
                        KeyError::IncorrectKey => tell_user!(ctx.writer, "You lack the right key to lock this entrace.\n"),
                        KeyError::Jammed => tell_user!(ctx.writer, "Ah bugger, the lock's jammed!\n"),
                        KeyError::NotLockable => tell_user!(ctx.writer, "Uh, you have no clue how to lock that…\n"),
                    }
                }
            } else {
                tell_user!(ctx.writer, "In theory, closing '{}' might work… if it was here, but it isn't.\n", dir);
            }
        });

        equalize_opposite_exit_state!(equalize, ctx, r_id, equalize_exit);
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
