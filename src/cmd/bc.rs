use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, show_help_if_needed, traits::Description, util::{comm::Channel, Broadcast}, validate_builder};

pub struct BcCommand;

#[async_trait]
impl Command for BcCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_builder!(ctx);
        show_help_if_needed!(ctx, "bc");

        let _ = ctx.tx.send(Broadcast::Channel { channel: Channel::Builder, message: ctx.args.into(), from_player: ctx.player.read().await.id().into() });
    }
}
