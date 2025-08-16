use async_trait::async_trait;
use crate::{cmd::{help::HelpCommand, look::LookCommand, Command, CommandCtx}, do_in_current_room, resume_game, tell_user, util::direction::Direction, ClientState};

pub struct GotoCommand;

/// Translocate player to some other spot in the world.
#[async_trait]
impl Command for GotoCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if ctx.args.is_empty() {
            ctx.args = "goto";
            let help = HelpCommand;
            return help.exec(ctx).await;
        }

        let exit: Result<Direction, _> = ctx.args.try_into();
        if exit.is_err() {
            tell_user!(ctx.writer, "Unknown direction. Use one of:\n{}\n", goto_directions());
            resume_game!(ctx);
        }
        
        let exit = exit.unwrap();

        // See if room has corresponding exit...
        do_in_current_room!(ctx, |room|{
            if let Some(droom_name) = room.read().await.exits.get(&exit) {
                if ctx.world.read().await.rooms.get(droom_name.as_str()).is_some() {
                    ctx.player.write().await.location = droom_name.clone();
                    let cmd = LookCommand;
                    cmd.exec(ctx).await;
                } else {
                    log::warn!("Room error: access to '{}' from '{}' is dysfunctional!", &droom_name, room.read().await.id);
                    tell_user!(ctx.writer, "You could've sworn there is something that way, but there isn't...\n");
                }
            } else {
                tell_user!(ctx.writer, "Cannot go that way …");
            }
        });

        resume_game!(ctx);
    }
}

fn goto_directions() -> String {r#"

    North, East, South, West,
    NorthEast, NorthWest, SouthEast, SouthWest,
    Up, Down."#.to_string()}

#[cfg(test)]
mod goto_tests {
    use std::{net::{IpAddr, Ipv4Addr}, str::FromStr, sync::Arc};

    use tokio::{io::{AsyncBufReadExt, AsyncReadExt, BufReader, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::{broadcast, RwLock}};

    use crate::{player::Player, world::{area::Area, room::Room, World}};

    use super::*;

    #[tokio::test]
    async fn go_a_to_b() {
        let _ = env_logger::try_init();
        log::info!("Preparing the stage …");
        let w = Arc::new(RwLock::new(World::blank()));
        let a = Arc::new(RwLock::new(Area::blank()));
        w.write().await.areas.insert("root".to_string(), a);
        {
            let w = w.read().await;
            let mut a = w.areas.get("root").unwrap().write().await;
            
            let r = Arc::new(RwLock::new(Room::blank(Some("void"))));
            r.write().await.description = "Alpha".to_string();
            r.write().await.exits.insert(Direction::East, "clearing".into());
            a.rooms.insert("void".to_string(), r);
            
            let r = Arc::new(RwLock::new(Room::blank(Some("clearing"))));
            r.write().await.description = "Omega".to_string();
            r.write().await.exits.insert(Direction::West, "void".into());
            a.rooms.insert("clearing".to_string(), r);
        }
        log::info!("World staged.");

        let p = Arc::new(RwLock::new(Player::new("ani")));
        p.write().await.location = "void".to_string();

        let ip = IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap());
        w.write().await.players.insert(ip.clone(), p);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, _) = broadcast::channel::<String>(1);

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
            let player_arc = w.read().await.players.get(&addr.ip()).unwrap().clone();

            // Handle "look" command
            server_reader.read_line(&mut line).await.unwrap();
            log::info!("client sent: \"{}\"", line);
            let ctx = CommandCtx {
                player: player_arc.clone(),
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
