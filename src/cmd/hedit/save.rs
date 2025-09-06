use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use crate::{cmd::{hedit::HeditState, Command, CommandCtx}, tell_user, traits::save::{DoesSave, SaveError}, validate_builder};

pub struct SaveCommand;

#[async_trait]
impl Command for SaveCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_builder!(ctx);
        
        if let Some(ref mut hs) = ctx.player.write().await.hedit {
            let g = hs.save().await;
            if let Err(_) = g {
                tell_user!(ctx.writer, "Oops?\n");
            } else {
                // Escalate changes to World itself.
                let mut w = ctx.world.write().await;
                if let Some(orig) = &hs.original {
                    for id in &orig.read().await.aliases {
                        w.help.remove(id);
                    }
                }
                let lock = Arc::new(RwLock::new(hs.entry.clone()));
                hs.original = Some(lock.clone());
                for id in &hs.entry.aliases {
                    w.help.insert(id.clone(), lock.clone());
                }
                tell_user!(ctx.writer, "Edits saved.\n");
            }
        } else {
            tell_user!(ctx.writer, "Nothing to save here…\n");
        }
    }
}

#[async_trait]
impl DoesSave for HeditState {
    async fn save(&mut self) -> Result<(), SaveError> {
        let _ = self.entry.save().await?;
        self.dirty = false;
        Ok(())
    }
}
