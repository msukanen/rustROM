use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, show_help_if_needed, util::Broadcast, validate_admin};

pub struct ForceCommand;

#[async_trait]
impl Command for ForceCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_admin!(ctx);
        show_help_if_needed!(ctx, "force");

        let force = Broadcast::Force { message: "look".into(), to_player: None, from_player: "«test»".to_string().into() };
        let _ = ctx.tx.send(force);
    }
}
