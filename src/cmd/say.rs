use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, resume_game, traits::Description, util::BroadcastMessage, ClientState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Subtype {
    Say,
    Ask,
    Exclaim,
}

pub struct SayCommand;

#[async_trait]
impl Command for SayCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if !ctx.args.is_empty() {
            let p = ctx.player.read().await;
            let message = format!("\n<c blue>[<c cyan>{}</c>]</c> says: {}\n", p.id(), ctx.args);
            let from_player = p.id().into();
            let room_id = p.location.clone();
            drop(p);
            ctx.tx.send(BroadcastMessage::Say { subtype: Some(Subtype::Say), room_id, message, from_player }).unwrap();
        }
        resume_game!(ctx);
    }
}
