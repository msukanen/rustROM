use std::{fmt::Display, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;
use crate::{cmd::{look::LookCommand, Command, CommandCtx}, player::Player, show_help, tell_user, traits::{save::DoesSave, Identity}, validate_admin, world::SharedWorld};

pub struct TranslocateCommand;

#[derive(Debug)]
pub enum TranslocationError {
    SourceNotFound,
    TargetNotFound,
    PlayerNotFound,
    NoMoveRequired,
}

impl std::error::Error for TranslocationError {}
impl Display for TranslocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: refine
        match self {
            Self::PlayerNotFound => write!(f, "Cannot translocate a non-existing entity"),
            Self::SourceNotFound => write!(f, "Not fatal, but notable. Source room not found"),
            Self::TargetNotFound => write!(f, "Target room does not exist?"),
            Self::NoMoveRequired => write!(f, "Source == Target. No move required."),
        }
    }
}

/// Translocate player to some other spot in the world.
#[async_trait]
impl Command for TranslocateCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_admin!(ctx);

        let (who, where_to) = {
            let args: Vec<&str> = ctx.args.splitn(2, ' ').collect();
            if args.len() < 2 {
                show_help!(ctx, "translocate");
            }
            (args[0], args[1])
        };

        if ctx.world.read().await.rooms.get(where_to).is_none() {
            tell_user!(ctx.writer, "No such room exists.\n");
            return;
        }

        // Who's being translocated?
        match who {
            "self" => {
                let source = ctx.player.read().await.location.clone();
                let _ = translocate(ctx.world, Some(source), where_to.into(), ctx.player.clone()).await;
                let look = LookCommand;
                look.exec({ctx.args = ""; ctx}).await;
            },
            _ => {
                let other = ctx.world.read().await.find_player(who);
                if let Some(found) = other {
                        log::info!("Translocating other player, '{}'", found.read().await.id());
                        let source = found.read().await.location.clone();
                        let _ = translocate(ctx.world, Some(source), where_to.into(), found.clone()).await;
/* TODO: convert this into "scry" command for later:
                        let you = ctx.player.clone();
                         ctx.player = found.clone();
                        let look = LookCommand;
                        look.exec({ctx.args = ""; ctx}).await;
                        ctx.player = you;
 */                } else {
                    tell_user!(ctx.writer, "Could not locate '{}'", who);
                }
            }
        }
    }
}

/// Translocate given player to another place in another time... or so.
/// 
/// If [Player] already *is* at the target and/or target is the same as source,
/// nothing will be done, of course.
/// 
/// # Arguments
/// - `world`— the [SharedWorld] itself.
/// - `source`— source room name, optional (to a degree).
/// - `target`— target room name - mandatory, naturally.
/// - `player`— the [Player] to be hauled over.
/// 
/// # Returns
/// - `Ok(None)` if everything went smooth.
/// - `Ok(SourceNotFound)` if source room was not found, but translocation itself still succeeded.
/// - `Ok(NoMoveRequired)` if `source==target`.
/// - `Err(TranslocationError)` if something went truly awry.
#[must_use = "Result must be used to ensure data/world integrity."]
pub(crate) async fn translocate(
    world: &SharedWorld,
    source: Option<String>,
    target: String,
    player: Arc<RwLock<Player>>
) -> Result<Option<TranslocationError>, TranslocationError> {
    let mut is_same = false;
    if let Some(source) = &source {
        is_same = source.eq(&target)
    }

    // Handle TARGET first...
    {
        let w = world.write().await;
        if let Some(r) = w.rooms.get(&target) {
            if r.write().await.add_player(&player).await {
                let r_id = r.read().await.id().to_string();
                let mut p = player.write().await;
                if !is_same && p.location != r_id {
                    p.location = r_id.clone();
                    p.inc_act_count();
                }
            }
        } else {
            log::error!("Target room '{}' not found!", target);
            return Err(TranslocationError::TargetNotFound);
        }
    }

    // Handle SOURCE second...
    let mut ok_err = None;
    if let Some(source) = source {
        if is_same {
            log::trace!("Skipping source extraction - target == source");
            return Ok(Some(TranslocationError::NoMoveRequired));
        }
        
        let w = world.write().await;
        if let Some(r) = w.rooms.get(&source) {
            r.write().await.remove_player(&player).await;
        } else {
            log::info!("Source room '{}' not found for translocation. Player '{}' still successfully translocated.", source, player.read().await.id());
            ok_err = Some(TranslocationError::SourceNotFound)
        }
    } else {
        log::trace!("Player '{}' successfully translocated to safety from The Void.", player.read().await.id());
        ok_err = Some(TranslocationError::SourceNotFound)
    }

    Ok(ok_err)
}
