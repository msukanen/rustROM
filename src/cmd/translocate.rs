use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, mob::{core::IsMob, stat::StatValue}, resume_game, tell_user, tell_user_unk, ClientState};

pub struct TranslocateCommand;

/// Translocate player to some other spot in the world.
#[async_trait]
impl Command for TranslocateCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if !ctx.player.read().await.access.is_admin() {
            tell_user_unk!(ctx.writer);
            resume_game!(ctx);
        }

        let args: Vec<&str> = ctx.args.splitn(2, ' ').collect();
        if args.len() < 2 {
            translocate_usage(ctx).await;
            resume_game!(ctx);
        }

        // Who's being translocated?
        match args[0] {
            "self" => {},
            _ => {}
        }

        let loc: Vec<&str> = args[1].split_terminator(&['.', ' ']).collect();
        if loc.len() < 2 {
            translocate_usage(ctx).await;
            resume_game!(ctx);
        }
        let area = loc[0];
        let room = loc[1];
        

        resume_game!(ctx);
    }
}

async fn translocate_usage(ctx: &mut CommandCtx<'_>) {
    tell_user!(ctx.writer, "\
'translocate' is used to transport a player (self or otherwise) to another \
(existing) location in the world.  The command, obviously, fails if target \
location does not exist.\n
\n
usage:  translocate self|TARGET AREA[.]ROOM");
}
