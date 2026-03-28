//! Emote something.
use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, show_help_if_needed, tell_user, traits::IdentityQuery, util::Broadcast};

pub struct EmoteCommand;

#[async_trait]
impl Command for EmoteCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        if ctx.args.is_empty() {
            return tell_user!(ctx.writer, "Feeling emotional today, aren't we?\n");
        }
        show_help_if_needed!(ctx, "emote");

        let p = ctx.player.read().await;
        let p_id = p.id();
        let message = format!("\n<c cyan>{p_id}</c> {}\n", ctx.args.trim());
        tell_user!(ctx.writer, message.strip_prefix('\n').unwrap_or(&message));
        let _ = ctx.tx.send(Broadcast::Say {
            subtype: None,
            room_id: p.location.clone(),
            message,
            from_player: p_id.into()
        });
    }
}
