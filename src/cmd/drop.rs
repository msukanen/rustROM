use async_trait::async_trait;
use crate::{cmd::{put::PutCommand, Command, CommandCtx}, show_help_if_needed};

pub struct DropCommand;

#[async_trait]
impl Command for DropCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "drop");
        let mut ctx = CommandCtx {
            player: ctx.player.clone(),
            state: ctx.state.clone(),
            world: ctx.world,
            tx: ctx.tx,
            writer: ctx.writer,
            args: &format!("{} ground", ctx.args)
        };
        let cmd = PutCommand;
        cmd.exec(&mut ctx).await
    }
}
