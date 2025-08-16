use std::{fmt::Display, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;
use crate::{cmd::{help::HelpCommand, look::LookCommand, Command, CommandCtx}, player::Player, resume_game, tell_user, tell_user_unk, traits::{save::DoesSave, Description}, validate_admin, world::{room::Room, SharedWorld}, ClientState};

pub struct TranslocateCommand;

#[derive(Debug)]
pub enum TranslocationError {
    SourceNotFound,
    TargetNotFound,
    PlayerNotFound,
}

impl std::error::Error for TranslocationError {}
impl Display for TranslocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: refine
        match self {
            Self::PlayerNotFound => write!(f, "Cannot translocate a non-existing entity"),
            Self::SourceNotFound => write!(f, "Not fatal, but notable. Source room not found"),
            Self::TargetNotFound => write!(f, "Target room does not exist?"),
        }
    }
}

/// Translocate player to some other spot in the world.
#[async_trait]
impl Command for TranslocateCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_admin!(ctx);

        let args: Vec<&str> = ctx.args.splitn(2, ' ').collect();
        if args.len() < 2 {
            let cmd = HelpCommand;
            return cmd.exec({ctx.args = "translocate"; ctx}).await;
        }

        let room = args[1];
        if ctx.world.read().await.rooms.get(room).is_none() {
            tell_user!(ctx.writer, "No such room exists.\n");
            resume_game!(ctx);
        }

        // Who's being translocated?
        match args[0] {
            "self" => {
                let source = ctx.player.read().await.location.clone();
                let _ = translocate(ctx.world, Some(source), room.into(), ctx.player.clone()).await;
                let look = LookCommand;
                look.exec({ctx.args = ""; ctx}).await;
            },
            _ => {
                todo!("Translocate another player.")
            }
        }

        resume_game!(ctx);
    }
}

/// Translocate given player to another place in another time... or so.
/// 
/// # Arguments
/// - `world`— the [World] itself.
/// - `source`— source room name, optional (to a degree).
/// - `target`— target room name - mandatory, naturally.
/// - `player`— the [Player] to be hauled over.
/// 
/// # Returns
/// - `Ok(None)` if everything went smooth.
/// - `Ok(SourceNotFound)` if source room was not found, but translocation itself still succeeded.
/// - `Err(TranslocationError)` if something went truly awry.
#[must_use = "Result must be used to ensure data/world integrity."]
pub(crate) async fn translocate(
    world: &SharedWorld,
    source: Option<String>,
    target: String,
    player: Arc<RwLock<Player>>
) -> Result<Option<TranslocationError>, TranslocationError> {
    // Handle TARGET first...
    {
        let w = world.write().await;
        if let Some(r) = w.rooms.get(&target) {
            let mut p = player.write().await;
            let mut r = r.write().await;
            r.players.insert(p.id().into(), Arc::downgrade(&player));
            p.location = r.id().into();
            // Save the player right here and right now after translocation.
            let _ = p.save().await;
            log::debug!("Added Player '{}' to target Room '{}'", p.id(), r.id());
        } else {
            log::error!("Target room '{}' not found!", target);
            return Err(TranslocationError::TargetNotFound);
        }
    }

    // Handle SOURCE second...
    let mut ok_err = None;
    if let Some(source) = source {
        let w = world.write().await;
        if let Some(r) = w.rooms.get(&source) {
            let mut r = r.write().await;
            let p = player.read().await;
            r.players.remove(p.id());
            log::debug!("Removed Player '{}' from source Room '{}'", p.id(), r.id());
        } else {
            log::debug!("Source room '{}' not found for translocation. Player '{}' still successfully translocated.", source, player.read().await.id());
            ok_err = Some(TranslocationError::SourceNotFound)
        }
    } else {
        log::debug!("Player '{}' successfully translocated to safety from The Void.", player.read().await.id());
        ok_err = Some(TranslocationError::SourceNotFound)
    }

    Ok(ok_err)
}
