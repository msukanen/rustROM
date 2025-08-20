use async_trait::async_trait;
use crate::{cmd::{ask::AskCommand, Command, CommandCtx}, show_help_if_needed, tell_user, traits::Identity, util::Broadcast};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Subtype {
    Say,
    Ask,
    Exclaim,
}

pub struct SayCommand;

#[async_trait]
impl Command for SayCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        if ctx.args.is_empty() {
            return tell_user!(ctx.writer, "You hear... crickets?\n");
        }
        show_help_if_needed!(ctx, "say");

        if !ctx.args.is_empty() {
            if ctx.args.ends_with('?') {
                let cmd = AskCommand;
                return cmd.exec(ctx).await;
            }
            let exlaim = ctx.args.ends_with('!');
            let p = ctx.player.read().await;
            let message = format!("\n<c blue>[<c cyan>{}</c>]</c> {}: {}\n", p.id(), if exlaim {"exclaims"} else {"says"}, ctx.args);
            let from_player = p.id().into();
            let room_id = p.location.clone();
            ctx.tx.send(Broadcast::Say { subtype: Some(Subtype::Say), room_id, message, from_player }).unwrap();
        }
    }
}
