use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{look::LookCommand, Command, CommandCtx}, do_in_current_room, resume_game, tell_command_usage, tell_user, util::direction::Direction, ClientState};

pub struct GotoCommand;

/// Translocate player to some other spot in the world.
#[async_trait]
impl Command for GotoCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if ctx.args.is_empty() {
            tell_command_usage!(ctx,
                "goto",
                "goes to places …",
                format!("{}{}<c green>Usage:</c> goto [DIR]", r#"
<c yellow>'goto'</c> is used to go to e.g. various directions, like:"#, goto_directions())
            );
        }

        let exit: Result<Direction, _> = ctx.args.try_into();
        if exit.is_err() {
            tell_user!(ctx.writer, "Unknown direction.\n\n{}\n", goto_directions());
            resume_game!(ctx);
        }
        
        let exit = exit.unwrap();

        // See if room has corresponding exit...
        do_in_current_room!(ctx, |room|{
            if let Some(droom_name) = room.read().await.exits.get(&exit) {
                // TODO: check that the room *actually* exists…
                ctx.player.write().await.location.room = droom_name.clone();
                let cmd = LookCommand;
                cmd.exec(ctx).await;
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
    Up, Down.

"#.to_string()
}

#[cfg(test)]
mod goto_tests {
    use std::{net::{IpAddr, Ipv4Addr}, str::FromStr, sync::Arc};

    use tokio::{io::AsyncReadExt, net::{TcpListener, TcpStream}, sync::{broadcast, RwLock}};

    use crate::{cmd::{goto::GotoCommand, Command, CommandCtx}, player::Player, world::{area::Area, room::Room, World}};

    #[tokio::test]
    async fn go_a_to_b() {
        let _ = env_logger::try_init();
        let w = Arc::new(RwLock::new(World::blank()));
        let a = Arc::new(RwLock::new(Area::blank()));
        w.write().await.areas.insert("root".to_string(), a);
        {
            let w = w.read().await;
            let mut a = w.areas.get("root").unwrap().write().await;
            
            let r = Arc::new(RwLock::new(Room::blank()));
            r.write().await.name = "void".to_string();
            r.write().await.description = "Alpha".to_string();
            a.rooms.insert("void".to_string(), r);
            
            let r = Arc::new(RwLock::new(Room::blank()));
            r.write().await.name = "clearing".to_string();
            r.write().await.description = "Omega".to_string();
            a.rooms.insert("clearing".to_string(), r);
        }

        let p = Arc::new(RwLock::new(Player::new("ani")));
        p.write().await.location.area = "root".to_string();
        p.write().await.location.room = "void".to_string();
        let ip = IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap());
        w.write().await.players.insert(ip.clone(), p);
        
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let client_task = tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.unwrap();
            let mut buffer = vec![];
            stream.read_to_end(&mut buffer).await.unwrap();
            String::from_utf8(buffer).unwrap()
        });
        let (server_socket, _) = listener.accept().await.unwrap();
        let (_, mut writer) = server_socket.into_split();
        let (tx, _) = broadcast::channel::<String>(1);
        let mut ctx = CommandCtx {
            player: w.read().await.players.get(&ip).unwrap().clone(),
            world: &w,
            writer: &mut writer,
            tx: &tx,
            args: "east"
        };
        let goto_cmd = GotoCommand;
        goto_cmd.exec(&mut ctx).await;
        let out = client_task.await.unwrap();
        assert!(out.contains("Omega"));
    }
}
