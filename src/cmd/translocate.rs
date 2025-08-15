use async_trait::async_trait;
use crate::{cmd::{look::LookCommand, Command, CommandCtx}, resume_game, tell_user, tell_user_unk, ClientState};

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

        let room = args[1];
        if ctx.world.read().await.rooms.get(room).is_none() {
            tell_user!(ctx.writer, "No such room exists.\n");
            resume_game!(ctx);
        }

        // Who's being translocated?
        match args[0] {
            "self" => {
                ctx.player.write().await.location = room.into();
                let look = LookCommand;
                look.exec({ctx.args = ""; ctx}).await;
            },
            _ => {
                todo!("Translocate another player.")
            }
        }

        resume_game!(ctx);
    }
}

async fn translocate_usage(ctx: &mut CommandCtx<'_>) {
    tell_user!(ctx.writer, r#"
<c yellow>'translocate'</c> is used to transport a player (self or otherwise) to another 
(existing) location in the world.  The command, obviously, fails if target 
location does not exist.

<c green>Usage:</c> <c yellow>translocate self [ROOM]</c>
       <c yellow>translocate [TARGET] [ROOM]
"#);}
