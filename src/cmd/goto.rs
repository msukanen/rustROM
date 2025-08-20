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
                if ctx.world.read().await.rooms.get(&exit.destination).is_some() {
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
mod goto_tests {
    use std::{net::SocketAddr, str::FromStr, sync::Arc};

    use tokio::{io::{AsyncBufReadExt, AsyncReadExt, BufReader, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::{broadcast, RwLock}};

    use crate::{player::Player, util::{Broadcast, ClientState}, world::{area::Area, room::{Exit, ExitState, Room}, World}};

    use super::*;

    #[tokio::test]
    async fn go_a_to_b() {
        let _ = env_logger::try_init();
        log::info!("Preparing the stage …");
        let w = Arc::new(RwLock::new(World::blank()));
        let a = Arc::new(RwLock::new(Area::blank()));
        w.write().await.areas.insert("root".to_string(), a);
        {
            let mut w = w.write().await;
            //let mut a = w.areas.get_mut("root").unwrap().write().await;
            
            let r = Arc::new(RwLock::new(Room::blank(Some("void"))));
            r.write().await.description = "Alpha".into();
            r.write().await.exits.insert(Direction::East, Exit { destination: "clearing".into(), state: ExitState::Open });
            //a.rooms.insert("void".into(), r.clone());
            w.rooms.insert("void".into(), r.clone());
            
            let r = Arc::new(RwLock::new(Room::blank(Some("clearing"))));
            r.write().await.description = "Omega".to_string();
            r.write().await.exits.insert(Direction::West, Exit { destination: "void".into(), state: ExitState::Open });
            w.rooms.insert("clearing".into(), r.clone());
            //a.rooms.insert("clearing".to_string(), r);
        }
        log::info!("World staged.");

        let p = Arc::new(RwLock::new(Player::new("ani")));
        p.write().await.location = "void".into();

        let ip = SocketAddr::from_str("127.0.0.1:12345").unwrap();
        w.write().await.players_by_sockaddr.insert(ip.clone(), p);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, _) = broadcast::channel::<Broadcast>(1);

        let client_task = tokio::spawn(async move {
            let (reader, mut writer) = TcpStream::connect(addr).await.unwrap().into_split();
            let mut reader = BufReader::new(reader);
            let mut buffer = String::new();

            // Send the "look" command
            writer.write_all(b"look\n").await.unwrap();
            // Send the "goto" command
            writer.write_all(b"goto east\n").await.unwrap();

            // Now, read all the output until the server closes the connection.
            reader.read_to_string(&mut buffer).await.unwrap();
            buffer
        });
        log::info!("Client task prepped…");

        // 4. Server-side logic now simulates the main loop for two commands.
        let server_task = tokio::spawn(async move {
            let (server_socket, _) = listener.accept().await.unwrap();
            let (server_reader, mut server_writer) = server_socket.into_split();
            let mut server_reader = BufReader::new(server_reader);
            let mut line = String::new();
            let player_arc = w.read().await.players_by_sockaddr.get(&addr).unwrap().clone();
            player_arc.write().await.push_state(ClientState::Playing);

            // Handle "look" command
            server_reader.read_line(&mut line).await.unwrap();
            log::info!("client sent: \"{}\"", line);
            let ctx = CommandCtx {
                player: player_arc.clone(),
                state: player_arc.read().await.state(),
                world: &w,
                tx: &tx,
                args: &line.trim(),
                writer: &mut server_writer };
            crate::cmd::parse_and_execute(ctx).await;
            
            // Handle "goto east" command
            line.clear();
            server_reader.read_line(&mut line).await.unwrap();
            let ctx = CommandCtx {
                player: player_arc.clone(),
                state: player_arc.read().await.state(),
                world: &w,
                tx: &tx,
                args: &line.trim(),
                writer: &mut server_writer };
            crate::cmd::parse_and_execute(ctx).await;
        }); // `server_socket` is dropped here, closing the connection.
        log::info!("Server task prepped…");

        // 5. Wait for the client task to finish and get the output.
        let (_, client_out) = tokio::join!(server_task, client_task);
        let output_string = client_out.unwrap();

        // 6. Assert that the output contains the description of BOTH rooms.
        log::info!("output_string = \n---\n{}\n---", output_string);
        assert!(output_string.contains("Alpha"));
        assert!(output_string.contains("Omega"));
    }
}
