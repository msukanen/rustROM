use std::sync::Arc;
use async_trait::async_trait;
use tokio::{net::tcp::OwnedWriteHalf, sync::{broadcast, RwLock}, io::AsyncWriteExt};
use crate::{player::Player, resume_game, tell_user, util::clientstate::EditorMode, world::SharedWorld, ClientState};

pub mod macros;
//--- 'mod' all the commands ---
mod quit;
mod say;
mod set;
mod look;
mod dig;
mod dmg;
mod translocate;
mod goto;
mod help;
mod r#return;
pub(crate) mod hedit;
mod redit;
mod abort;

/// Player locker.
type PlayerLock = Arc<RwLock<Player>>;

/// Command context for all the commands to chew on.
pub struct CommandCtx<'a> {
    pub player: PlayerLock,
    pub world: &'a SharedWorld,
    pub tx: &'a broadcast::Sender<String>,
    pub args: &'a str,
    pub writer: &'a mut OwnedWriteHalf,
}

/// Short cmd ctx for e.g. those specific helpers which never
/// need anything else than this particular triplet.
pub struct ShortCommandCtx<'a> {
    pub player: PlayerLock,
    pub world: &'a SharedWorld,
    pub writer: &'a mut OwnedWriteHalf,
}

impl <'a> CommandCtx<'a> {
    /// Get a [ShortCommandCtx] version of self.
    pub fn short_ctx(&mut self) -> ShortCommandCtx<'_> {
        ShortCommandCtx {
            player: self.player.clone(),
            world: self.world,
            writer: self.writer,
        }
    }
}

/// An async trait for all commands to obey.
#[async_trait]
pub trait Command: Send + Sync {
    /// Do something …
    /// 
    /// # Arguments
    /// - `ctx`— [CommandCtx]
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState;
}

// Get the COMMANDS hashmap …:
include!(concat!(env!("OUT_DIR"), "/commands.rs"));

/// Parses the player's input and executes the corresponding command.
/// 
/// # Arguments
/// - `ctx`– [CommandCtx], crafted in `main()` (usually).
pub async fn parse_and_execute<'a>(mut ctx: CommandCtx<'_>) -> ClientState {
    if ctx.args.is_empty() {// no need for whitespace check as input's already trimmed earlier.
        resume_game!(ctx);
    }

    let (command, args) = ctx.args.split_once(' ').unwrap_or((ctx.args, ""));
    ctx.args = args;
    
    let table = match ctx.player.read().await.state() {
        ClientState::Playing => &COMMANDS,
        ClientState::Editing { ref mode, .. } => match mode {
            EditorMode::Room { .. } => &REDIT_COMMANDS,
            EditorMode::Help { .. } => &HEDIT_COMMANDS,
        },
        _ => {// Should not happen, but ...
            log::error!("Player state '{:?}' invalid?", ctx.player.read().await.state());
            resume_game!(ctx);
        }
    };

    if let Some(cmd) = table.get(command.to_lowercase().as_str()) {
        cmd.exec(&mut ctx).await
    } else if let Some(cmd) = COMMANDS.get(command.to_lowercase().as_str()) {
        cmd.exec(&mut ctx).await
    } else {
        tell_user!(ctx.writer, "Huh?\n");
        ctx.player.read().await.state()
    }
}
