use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, mob::{core::IsMob, stat::StatValue}, resume_game, tell_user, tell_user_unk, ClientState};

pub struct TranslocateCommand;

/// Translocate player to some other spot in the world.
#[async_trait]
impl Command for TranslocateCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if !ctx.player.access.is_admin() {
            tell_user_unk!(ctx.writer);
            resume_game!(ctx);
        }

        let args: Vec<&str> = ctx.args.splitn(3, ' ').collect();
        if args.len() < 3 {
            tell_user!(ctx.writer, "\
'translocate' is used to transport a player (self or otherwise) to another\n\
(existing) location in the world.  The command, obviously, fails if target\n\
location does not exist.\n\
\n\
usage:  translocate self|TARGET AREA[.]ROOM
");
            resume_game!(ctx);
        }

        

        resume_game!(ctx);
    }
}
