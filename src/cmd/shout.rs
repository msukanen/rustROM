use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, resume_game, string::exclaim::exclaim_if_needed, traits::Description, util::BroadcastMessage, ClientState};

pub struct ShoutCommand;

#[async_trait]
impl Command for ShoutCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if !ctx.args.is_empty() {
            let p = ctx.player.read().await;
            let message = format!("\n<c blue>[<c cyan>{}</c>]</c> shouts: {}\n", p.id(), exclaim_if_needed(ctx.args));
            let from_player = p.id().into();
            let room_id = p.location.clone();
            drop(p);
            ctx.tx.send(BroadcastMessage::Shout { room_id, message, from_player }).unwrap();
        }
        resume_game!(ctx);
    }
}
