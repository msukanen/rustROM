use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, show_help_if_needed, string::exclaim::exclaim_if_needed, traits::Description, util::Broadcast};

pub struct ShoutCommand;

#[async_trait]
impl Command for ShoutCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "shout");
        
        if !ctx.args.is_empty() {
            let p = ctx.player.read().await;
            let message = format!("\n<c blue>[<c cyan>{}</c>]</c> shouts: {}\n", p.id(), exclaim_if_needed(ctx.args));
            let from_player = p.id().into();
            let room_id = p.location.clone();
            drop(p);
            ctx.tx.send(Broadcast::Shout { room_id, message, from_player }).unwrap();
        }
    }
}
