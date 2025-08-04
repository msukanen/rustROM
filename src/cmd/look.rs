use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, ClientState};

pub struct LookCommand;

#[async_trait]
impl Command for LookCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if let Some(area) = ctx.world.read().await.areas.get(&ctx.player.location.area) {
            if let Some(room) = area.read().await.rooms.get(&ctx.player.location.room) {
                tell_user!(ctx.writer, "LOOK\n\n{}\n", room.read().await.description());
            }
        } else {
            tell_user!(ctx.writer, "You see... nothing much else than a wall of white text on a dark surface?\n");
        }
        tell_user!(ctx.writer, "{}", ctx.prompt);
        resume_game!(ctx);
    }
}
