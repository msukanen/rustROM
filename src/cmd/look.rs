use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, ClientState};

pub(crate) struct LookCommand;

#[async_trait]
impl Command for LookCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if let Some(room) = ctx.world.read().await.find_room(ctx.player.location.area.as_str(), ctx.player.location.room.as_str()) {
            tell_user!(ctx.writer, format!("LOOK\n\n{}", room.description()));
        } else {
            tell_user!(ctx.writer, "You see... nothing much else than a wall of white text on a dark surface?\n");
        }
        resume_game!(ctx);
    }
}
