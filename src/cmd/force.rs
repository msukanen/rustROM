use async_trait::async_trait;
use crate::{cmd::{help::HelpCommand, Command, CommandCtx}, util::Broadcast, validate_admin};

pub struct ForceCommand;

#[async_trait]
impl Command for ForceCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_admin!(ctx);

        if ctx.args.is_empty() || ctx.args.starts_with('?') {
            let cmd = HelpCommand;
            return cmd.exec({ctx.args = "force"; ctx}).await;
        }

        let force = Broadcast::Force { message: "look".into(), to_player: None, from_player: "«test»".to_string().into() };
        let _ = ctx.tx.send(force);
    }
}
