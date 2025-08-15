use async_trait::async_trait;
use crate::{cmd::{hedit::HeditState, Command, CommandCtx}, resume_game, tell_user, traits::save::{DoesSave, SaveError}, validate_builder, ClientState};

pub struct SaveCommand;

#[async_trait]
impl Command for SaveCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);
        
        if let Some(ref mut hs) = ctx.player.write().await.hedit {
            let g = hs.save().await;
            if let Err(_) = g {
                log::error!("Error saving '{}'", ctx.player.read().await.hedit.as_ref().unwrap().lock.read().await.id);
                tell_user!(ctx.writer, "Oops?\n");
            } else {
                tell_user!(ctx.writer, "Edits saved.\n");
            }
        } else {
            tell_user!(ctx.writer, "Nothing to save here...\n");
        }
        resume_game!(ctx);
    }
}

#[async_trait]
impl DoesSave for HeditState {
    async fn save(&mut self) -> Result<(), SaveError> {
        let _ = self.lock.write().await.save().await?;
        self.dirty = false;
        Ok(())
    }
}
