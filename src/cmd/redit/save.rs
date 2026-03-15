//! REdit/'save' — save [Room] edits.
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{cmd::{Command, CommandCtx}, tell_user, traits::{Identity, save::DoesSave}, validate_builder, world::room::Room};

pub struct SaveCommand;

#[async_trait]
impl Command for SaveCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_builder!(ctx);

        let mut p = ctx.player.write().await;
        if let Some(ref mut ed) = p.redit {
            let room: &mut Room = &mut ed.entry;
            if let Err(e) = room.save().await {
                log::error!("FATAL: save error '{e:?}'");
                tell_user!(ctx.writer, "Something went awry…\n");
                return;
            }
            ed.dirty = false;
            // toss into the wild!
            let mut w = ctx.world.write().await;
            if let Some(orig) = w.rooms.get(room.id()) {
                let mut lock = orig.write().await;
                lock.shallow_copy(room);
            } else {
                w.rooms.insert(room.id().into(), Arc::new(RwLock::new(room.clone())));
            }

            tell_user!(ctx.writer, "Edits stored.\n");
        } else {
            log::debug!("Where'd the stored REdit state go for '{}'?!", p.id());
            tell_user!(ctx.writer, "You could've sworn you were editing a room, but…\n");
        }
    }
}
