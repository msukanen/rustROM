use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, ClientState, world::room::{Room, RoomError}};

pub struct DigCommand;

#[async_trait]
impl Command for DigCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
/*         let room = current_room!(ctx);
        if ctx.world.write().await.transfer_to_safety(&mut ctx, &room) {
            tell_user!(ctx.writer, "You were in limbo.\nAborting command…\nBending space and time…\n{}", ctx.prompt);
            resume_game!(ctx);
        }
 */        tell_user!(ctx.writer, "Diggy-diggy-hole toward {:?}", "x");
        resume_game!(ctx);
    }
}
