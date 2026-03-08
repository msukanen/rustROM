use async_trait::async_trait;
use crate::{AUTOSAVE_QUEUE_INTERVAL, cmd::{Command, CommandCtx}, string::robust_parse::RobustParse, tell_user, validate_admin};

pub struct AsqiCommand;

/// The cryptic command 'asqi' ...
/// 
/// Real-time auto-save queue interval tuning.
#[async_trait]
impl Command for AsqiCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_admin!(ctx);
        // For sake of sanity, we clamp the seconds into 1..600 range.
        let duration = ctx.args.robust_parse::<u64>();
        if duration.is_err() {
            tell_user!(ctx.writer, "'{}' is not a number representation I recognize... Sorry, Dave, cannot let you pass.", ctx.args);
            return;
        }
        let old = *AUTOSAVE_QUEUE_INTERVAL.read().await;
        *AUTOSAVE_QUEUE_INTERVAL.write().await = duration.unwrap();
        let duration = *AUTOSAVE_QUEUE_INTERVAL.read().await;
        tell_user!(ctx.writer, "Auto-save queue interval changed from {}s to {}s.\n", old, duration);
    }
}
