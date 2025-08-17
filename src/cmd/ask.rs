use async_trait::async_trait;
use crate::{cmd::{say::Subtype, Command, CommandCtx}, resume_game, traits::Description, util::BroadcastMessage, ClientState};

pub struct AskCommand;

#[async_trait]
impl Command for AskCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if !ctx.args.is_empty() {
            let p = ctx.player.read().await;
            let message = format!("\n<c blue>[<c cyan>{}</c>]</c> asks: {}{}\n", p.id(), ctx.args, if ctx.args.ends_with('?') {""} else {"?"});
            let from_player = p.id().into();
            let room_id = p.location.clone();
            drop(p);
            ctx.tx.send(BroadcastMessage::Say { subtype: Some(Subtype::Ask), room_id, message, from_player }).unwrap();
        }
        resume_game!(ctx);
    }
}
