use async_trait::async_trait;
use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf, sync::broadcast};

use crate::{player::save::Player, tell_user, world::SharedWorld, ClientState};

//--- 'mod' all the commands ---
mod quit;
mod say;
mod set;
mod look;

/// Command context for all the commands to chew on.
pub struct CommandCtx<'a> {
    pub player: Player,
    pub world: &'a SharedWorld,
    pub tx: &'a broadcast::Sender<String>,
    pub args: &'a str,
    pub writer: &'a mut OwnedWriteHalf,
    pub prompt: &'a str,
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
        tell_user!(writer, format!("Huh?\n{}", prompt));
        ClientState::Playing(player.clone())
    }
}

/// Quip to user about "unknown" command.  This is also used to mask
/// admin-only commands behind obscurity.
#[macro_export]
macro_rules! tell_unknown_command {
    ($ctx:expr) => {
        tell_user!($ctx.writer, format!("Huh?\n{}", $ctx.prompt));
    };
}

/// Shorthand for returning from a variety of commands into 'playing'
/// state of existence.
#[macro_export]
macro_rules! resume_game {
    ($ctx:expr) => {
        return ClientState::Playing($ctx.player.clone());
    };
}
