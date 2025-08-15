use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx, ShortCommandCtx}, do_in_current_room, resume_game, tell_user, traits::Description, ClientState};

pub struct LookCommand;

#[async_trait]
impl Command for LookCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        let mut ctx = ctx.short_ctx();
        look_at_current_room(&mut ctx).await
    }
}

/// The looking glass… used by e.g. 'look' command, etc.
pub async fn look_at_current_room(ctx: &mut ShortCommandCtx<'_>) -> ClientState {
    do_in_current_room!(ctx, |room| {
        let r = room.read().await;
        let mut desc = format!(
            "<c yellow>{}</c>\n\n{}\n\n",
            r.title(),
            r.description()
        );

        if !r.exits.is_empty() {
            desc.push_str("<c green>Exits:</c> ");
            let exits: Vec<String> = r.exits.keys().map(|d| format!("{:?}", d).to_lowercase()).collect();
            desc.push_str(&exits.join(", "));
            desc.push_str("\n\n");
        }
        tell_user!(ctx.writer, &desc);
    } otherwise {
        tell_user!(ctx.writer, "You see... nothing much else than a wall of white text on a dark surface?\n");
    });
    resume_game!(ctx);
}
