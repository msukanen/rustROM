//! <HEdit> 'save' subcommand.
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use crate::{help_reg_lock, cmd::{Command, CommandCtx, hedit::HeditState}, tell_user, traits::save::{DoesSave, SaveError}, validate_builder};

pub struct SaveCommand;

#[async_trait]
impl Command for SaveCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_builder!(ctx);
        
        if let Some(ref mut ed) = ctx.player.write().await.hedit {
            let g = ed.save().await;
            if let Err(_) = g {
                tell_user!(ctx.writer, "Oops?\n");
                return;
            }

            // Escalate changes to World itself.
            //---
            let mut h = help_reg_lock!(write);
            if let Some(original) = &ed.original {
                let original = original.read().await;
                // erase original itself:
                h.0.remove(&original.id);
                // erase existing aliases from global:
                for id in &original.aliases {
                    h.1.remove(id);
                }
            }

            let new = Arc::new(RwLock::new(ed.entry.clone()));
            ed.original = Some(new.clone());
            h.0.insert(ed.entry.id.clone(), new.clone());
            // insert new aliases:
            for id in &ed.entry.aliases {
                h.1.insert(id.clone(), ed.entry.id.clone());
            }
            tell_user!(ctx.writer, "Edits saved.\n");
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
