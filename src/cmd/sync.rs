//! Forced [World] synchronization… (onto disk).

use async_trait::async_trait;

use crate::{cmd::{Command, CommandCtx}, tell_user, traits::save::DoesSave, validate_admin};
pub(crate) struct SyncCommand;

#[async_trait]
impl Command for SyncCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_admin!(ctx);

        let mut w = ctx.world.write().await;
        if let Err(e) = w.save().await {
            tell_user!(ctx.writer, "Something is horribly wrong!\nSystem shutting down to prevent damage!\nCode most likely needs fixing ASAP…\n");
            log::error!("FATAL: World save error! {e:?}");
            panic!("World save error! {e:?}");
        }
    }
}
