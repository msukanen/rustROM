use async_trait::async_trait;
use crate::{cmd::{translocate::translocate, Command, CommandCtx}, cmd_exec, do_in_current_room, show_help_if_needed, tell_user, traits::Identity, util::direction::Direction};

pub struct GotoCommand;

/// Go some direction (or portal, etc.).
#[async_trait]
impl Command for GotoCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "goto");

        let exit: Result<Direction, _> = ctx.args.try_into();
        if exit.is_err() {
            tell_user!(ctx.writer, "Unknown direction.\n");
            return cmd_exec!(ctx, help, "q dir");
        }
        
        let exit = exit.unwrap();

        // See if room has corresponding exit...
        let mut do_translocate_to = None;
        do_in_current_room!(ctx, |room|{
            if let Some(exit) = room.read().await.exits.get(&exit).cloned() {
                if exit.is_closed() {
                    tell_user!(ctx.writer, "Well… the way to {} is {}. Try open it, maybe?\n", exit.destination, exit.state);
                } else if ctx.world.read().await.rooms.get(&exit.destination).is_some() {
                    // translocate afterwards so that all currently held locks are released first.
                    do_translocate_to = Some((room.read().await.id().to_string(), exit.destination.clone()));
                } else {
                    log::warn!("Room error: access to '{}' from '{}' is dysfunctional!", &exit.destination, room.read().await.id);
                    tell_user!(ctx.writer, "You could've sworn there is something that way, but there isn't...\n");
                }
            } else {
                tell_user!(ctx.writer, "You have no idea how to go there … Find another route?\n");
            }
        });

        if let Some((source, destination)) = do_translocate_to {
            let _ = translocate(&ctx.world, Some(source), destination, ctx.player.clone()).await;
            cmd_exec!(ctx, look);
        }
    }
}

#[cfg(test)]
mod cmd_goto_tests {
    use std::sync::Arc;
    use tokio::{io::{AsyncBufReadExt, AsyncReadExt, BufReader, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::{broadcast, RwLock}};
    use crate::{async_client_for_tests, async_server_for_tests, player::Player, player_and_listener_for_tests, string::ansi::AntiAnsi, util::{Broadcast, ClientState}, world::{World, area::Area, exit::{state::ExitState, *}, room::Room}, world_for_tests};
    use super::*;

    #[tokio::test]
    async fn go_a_to_b() {
        let _ = env_logger::try_init();

        log::info!("Preparing the stage …");

        // stage the World…
        let w = world_for_tests!();
        let (p, listener, addr, tx) = player_and_listener_for_tests!();
        let client_task = async_client_for_tests!(addr, "look", "goto east");
        let server_task = async_server_for_tests!(w, listener, tx, addr, p, 2);

        // wait for the client task to finish and get the output…
        let (_, client_out) = tokio::join!(server_task, client_task);
        let output_string = client_out.unwrap();
        let output_string = output_string.strip_ansi();

        // assert that the output contains the description of BOTH rooms.
        assert!(output_string.contains("Alpha"));
        assert!(output_string.contains("Omega"));
        assert!(output_string.contains("[ani]"));
    }

    #[tokio::test]
    async fn try_go_a_to_b_via_closed_exit() {
        let _ = env_logger::try_init();

        log::info!("Preparing the stage …");

        // stage the World…
        let w = world_for_tests!();
        // close void's east exit…
        {
            let mut lock = w.write().await;
            if let Some(room) = lock.rooms.get_mut("void") {
                room.write().await.set_exit_state(Direction::East, ExitState::Closed{key_id:None,jam:None});
            }
        }

        let (p, listener, addr, tx) = player_and_listener_for_tests!();
        let client_task = async_client_for_tests!(addr, "look", "goto east");
        let server_task = async_server_for_tests!(w, listener, tx, addr, p, 2);

        // wait for the client task to finish and get the output…
        let (_, client_out) = tokio::join!(server_task, client_task);
        let output_string = client_out.unwrap();
        let output_string = output_string.strip_ansi();

        // assert that destination is unreachable.
        assert!(output_string.contains("Alpha"));
        assert!(output_string.contains("the way to clearing is closed"));
    }
}
