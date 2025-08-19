use async_trait::async_trait;
use crate::{cmd::{say::Subtype, Command, CommandCtx}, show_help_if_needed, traits::Description, util::Broadcast};

pub struct AskCommand;

#[async_trait]
impl Command for AskCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "ask");

        let p = ctx.player.read().await;
        let message = format!("\n<c blue>[<c cyan>{}</c>]</c> asks: {}{}\n", p.id(), ctx.args, if ctx.args.ends_with('?') {""} else {"?"});
        let from_player = p.id().into();
        let room_id = p.location.clone();
        ctx.tx.send(Broadcast::Say { subtype: Some(Subtype::Ask), room_id, message, from_player }).unwrap();
    }
}
