use std::sync::Arc;
use async_trait::async_trait;
use tokio::{net::tcp::OwnedWriteHalf, sync::{broadcast, RwLock}};
use crate::{player::Player, tell_user, util::{clientstate::EditorMode, Broadcast}, world::SharedWorld, ClientState};

pub mod macros;
//--- 'mod' all the commands ---
mod quit;

pub(crate) mod say;
pub(crate) mod ask;
mod set;
mod look;
mod dig;
mod dmg;
pub(crate) mod translocate;
mod goto;
mod help;
mod r#return;
pub(crate) mod hedit;
pub(crate) mod redit;
mod abort;
mod shout;
mod badname;
pub(crate) mod force;
mod bc;
mod ac;

/// Player locker.
type PlayerLock = Arc<RwLock<Player>>;

/// Command context for all the commands to chew on.
pub struct CommandCtx<'a> {
    pub player: PlayerLock,
    pub state: ClientState,
    pub world: &'a SharedWorld,
    pub tx: &'a broadcast::Sender<Broadcast>,
    pub args: &'a str,
    pub writer: &'a mut OwnedWriteHalf,
}

/// An async trait for all commands to obey.
#[async_trait]
pub trait Command: Send + Sync {
    /// Do something …
    /// 
    /// # Arguments
    /// - `ctx`— [CommandCtx]
    async fn exec(&self, ctx: &mut CommandCtx<'_>);
}

// Get the COMMANDS hashmap …:
include!(concat!(env!("OUT_DIR"), "/commands.rs"));

/// Parses the player's input and executes the corresponding command.
/// 
/// # Arguments
/// - `ctx`– [CommandCtx], crafted in `main()` (usually).
pub async fn parse_and_execute<'a>(mut ctx: CommandCtx<'_>) -> ClientState {
    let state = ctx.player.read().await.state();
    
    if ctx.args.is_empty() {// no need for whitespace check as input's already trimmed earlier.
        return state;
    }

    let (command, args) = ctx.args.split_once(' ').unwrap_or((ctx.args, ""));
    ctx.args = args;
    
    let table = match state {
        ClientState::Playing => &COMMANDS,
        ClientState::Editing { ref mode, .. } => match mode {
            EditorMode::Room { .. } => &REDIT_COMMANDS,
            EditorMode::Help { .. } => &HEDIT_COMMANDS,
        },
        _ => {// Should not happen, but ...
            log::error!("Player state '{:?}' invalid?", state);
            return state;
        }
    };

    if let Some(cmd) = table.get(command.to_lowercase().as_str()) {
        cmd.exec(&mut ctx).await;
    } else if let Some(cmd) = COMMANDS.get(command.to_lowercase().as_str()) {
        cmd.exec(&mut ctx).await;
    } else {
        tell_user!(ctx.writer, "Huh?\n");
    }
    ctx.player.read().await.state()
}
