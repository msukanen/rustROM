use async_trait::async_trait;
use crate::{cmd::{help::HelpCommand, say::Subtype, Command, CommandCtx}, traits::Description, util::BroadcastMessage};

pub struct AskCommand;

#[async_trait]
impl Command for AskCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        if ctx.args.is_empty() || ctx.args.starts_with('?') {
            let cmd = HelpCommand;
            return cmd.exec({ctx.args = "ask"; ctx}).await;
        }

        let p = ctx.player.read().await;
        let message = format!("\n<c blue>[<c cyan>{}</c>]</c> asks: {}{}\n", p.id(), ctx.args, if ctx.args.ends_with('?') {""} else {"?"});
        let from_player = p.id().into();
        let room_id = p.location.clone();
        ctx.tx.send(BroadcastMessage::Say { subtype: Some(Subtype::Ask), room_id, message, from_player }).unwrap();
    }
}
