use async_trait::async_trait;
use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf, sync::broadcast};

use crate::{player::save::Player, world::SharedWorld, ClientState};

pub struct CommandCtx<'a> {
    pub player: Player,
    pub world: &'a SharedWorld,
    pub tx: &'a broadcast::Sender<String>,
    pub args: &'a str,
    pub writer: &'a mut OwnedWriteHalf,
    pub prompt: &'a str,
}

#[async_trait]
pub trait Command: Send + Sync {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState;
}

pub mod quit;
pub mod say;

include!(concat!(env!("OUT_DIR"), "/commands.rs"));

/// Parses the player's input and executes the corresponding command.
pub async fn parse_and_execute<'a>(
    player: Player,
    world: &'a SharedWorld,
    tx: &'a broadcast::Sender<String>,
    input: &'a str,
    writer: &'a mut OwnedWriteHalf,
    prompt: &'a str,
) -> ClientState {
    let (command, args) = input.split_once(' ').unwrap_or((input, ""));
    
    if let Some(cmd) = COMMANDS.get(command.to_lowercase().as_str()) {
        let mut ctx = CommandCtx {
            player,
            world,
            tx,
            args: args.trim(),
            writer,
            prompt,
        };
        cmd.exec(&mut ctx).await
    } else {
        writer.write_all(b"Huh?\n> ").await.unwrap();
        ClientState::Playing(player.clone())
    }
}
