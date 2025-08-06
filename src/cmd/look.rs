use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, do_in_current_room, resume_game, tell_user, traits::Description, ClientState};

pub struct LookCommand;

#[async_trait]
impl Command for LookCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        do_in_current_room!(ctx, |room| {
            let r = room.read().await;
            tell_user!(ctx.writer, "{}\n{}\n\n", r.title(), r.description());
        } otherwise {
            tell_user!(ctx.writer, "You see... nothing much else than a wall of white text on a dark surface?\n");
        });
        resume_game!(ctx);
    }
}
