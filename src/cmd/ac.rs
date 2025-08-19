use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, show_help_if_needed, traits::Description, util::{comm::Channel, Broadcast}, validate_admin};

pub struct AcCommand;

#[async_trait]
impl Command for AcCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_admin!(ctx);
        show_help_if_needed!(ctx, "ac");
        let _ = ctx.tx.send(Broadcast::Channel { channel: Channel::Admin, message: ctx.args.into(), from_player: ctx.player.read().await.id().into() });
    }
}
