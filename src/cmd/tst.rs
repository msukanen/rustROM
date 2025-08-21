use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, tell_user, validate_admin, AUTOSAVE_QUEUE_INTERVAL};

pub struct TstCommand;

#[async_trait]
impl Command for TstCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_admin!(ctx);
        let duration = ctx.args.parse::<u64>().unwrap();
        let old = *AUTOSAVE_QUEUE_INTERVAL.read().await;
        *AUTOSAVE_QUEUE_INTERVAL.write().await = duration;
        let duration = *AUTOSAVE_QUEUE_INTERVAL.read().await;
        tell_user!(ctx.writer, "Auto-save queue interval changed from {}s to {}s.\n", old, duration);
    }
}
