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
        let a = Arc::new(RwLock::new({
            let mut area = Area::blank();
            area.id = "area".into();
            area
        }));
        // stage the World…
        {
            let mut world_lock = w.write().await;
            
            // room #1
            let r = Arc::new(RwLock::new(Room::blank(Some("void"))));{
                let mut room_lock = r.write().await;
                room_lock.description = "Alpha".into();
                room_lock.exits.insert(Direction::East, Exit { destination: "clearing".into(), state: ExitState::Open });
                room_lock.parent_id = "area".into();
                room_lock.parent = Arc::downgrade(&a);
            }
            world_lock.rooms.insert("void".into(), r);
            
            // room #2
            let r = Arc::new(RwLock::new(Room::blank(Some("clearing"))));{
                let mut room_lock = r.write().await;
                room_lock.description = "Omega".to_string();
                room_lock.exits.insert(Direction::West, Exit { destination: "void".into(), state: ExitState::Open });
                room_lock.parent_id = "area".into();
                room_lock.parent = Arc::downgrade(&a);
            }
            world_lock.rooms.insert("clearing".into(), r);

            // put the area into play
            world_lock.areas.insert("root".to_string(), a);
            drop(world_lock);
        }
        log::info!("World staged.");

        let p = Arc::new(RwLock::new(Player::new("ani")));
        p.write().await.location = "void".into();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, _) = broadcast::channel::<Broadcast>(1);

        let client_task = tokio::spawn(async move {
            let (reader, mut writer) = TcpStream::connect(addr).await.unwrap().into_split();
            log::debug!("client_task: connected.");
            let mut reader = BufReader::new(reader);
            let mut buffer = String::new();

            // Send the "look" command
            writer.write_all(b"look\n").await.unwrap();
            // Send the "goto" command
            writer.write_all(b"goto east\n").await.unwrap();

            // Now, read all the output until the server closes the connection.
            reader.read_to_string(&mut buffer).await.unwrap();
            log::debug!("client_tast: server response|→{}←|", buffer);
            buffer
        });
        log::info!("Client task prepped…");

        // 4. Server-side logic now simulates the main loop for two commands.
        let server_task = tokio::spawn(async move {
            let (server_socket, client_addr) = listener.accept().await.unwrap();
            log::debug!("server_task: connection from {addr:?}");
            w.write().await.players_by_sockaddr.insert(client_addr, p);
            let (server_reader, mut server_writer) = server_socket.into_split();
            let mut server_reader = BufReader::new(server_reader);
            let mut line = String::new();
            log::debug!("server_task: player_arc?");
            let player_arc = w.read().await.players_by_sockaddr.get(&client_addr).unwrap().clone();
            log::debug!("server_task: player_arc ok");
            player_arc.write().await.push_state(ClientState::Playing);

            // Handle "look" command
            server_reader.read_line(&mut line).await.unwrap();
            log::info!("server_task: client cmd#1 \"{}\"", line.trim());
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
            log::info!("server_task: client cmd#2 \"{}\"", line.trim());
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

        // wait for the client task to finish and get the output…
        let (_, client_out) = tokio::join!(server_task, client_task);
        let output_string = client_out.unwrap();

        // de-grit output_string... (strip ANSI)
        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        let output_string = re.replace_all(&output_string, "");

        // assert that the output contains the description of BOTH rooms.
        assert!(output_string.contains("Alpha"));
        assert!(output_string.contains("Omega"));
        assert!(output_string.contains("[ani]"));
    }
}
