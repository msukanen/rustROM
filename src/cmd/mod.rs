use std::sync::Arc;
use async_trait::async_trait;
use tokio::{net::tcp::OwnedWriteHalf, sync::{broadcast, RwLock}, io::AsyncWriteExt};
use crate::{player::Player, tell_user, world::SharedWorld, ClientState};

pub mod macros;
//--- 'mod' all the commands ---
mod quit;
mod say;
mod set;
mod look;
mod dig;
mod dmg;
mod translocate;

/// Command context for all the commands to chew on.
pub struct CommandCtx<'a> {
    pub player: Player,
    pub world: &'a SharedWorld,
    pub tx: &'a broadcast::Sender<String>,
    pub args: &'a str,
    pub writer: &'a mut OwnedWriteHalf,
}

/// An async trait for all commands to obey.
#[async_trait]
pub trait Command: Send + Sync {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState;
}

// Get the COMMANDS hashmap …:
include!(concat!(env!("OUT_DIR"), "/commands.rs"));

/// Parses the player's input and executes the corresponding command.
/// 
/// # Arguments
/// - `player`— [Player], obviously.
/// - `world`— reference to the world itself. Seldom used, but one never knows…
/// - `tx`— global broadcast channel.
/// - `input`— whatever the user typed…
/// - `writer`— channel to deliver text to the user.
/// - `prompt`— prompt to show after command execution.
///             This may get overridden by specific commands.
pub async fn parse_and_execute<'a>(
    player: Player,
    world: &'a SharedWorld,
    tx: &'a broadcast::Sender<String>,
    input: &'a str,
    writer: &'a mut OwnedWriteHalf
) -> ClientState {
    if input.is_empty() {// no need for whitespace check as input's already trimmed earlier.
        return ClientState::Playing(player);
    }

    let (command, args) = input.split_once(' ').unwrap_or((input, ""));
    
    if let Some(cmd) = COMMANDS.get(command.to_lowercase().as_str()) {
        let mut ctx = CommandCtx {
            player,
            world,
            tx,
            args: args.trim(),
            writer,
        };
        cmd.exec(&mut ctx).await
    } else {
        tell_user!(writer, "Huh?\n");
        ClientState::Playing(player)
    }
}
