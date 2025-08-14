use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{help::HelpCommand, Command, CommandCtx}, resume_game, tell_user, util::direction::Direction, world::room::{Room, RoomError}, ClientState};

pub struct DigCommand;

#[async_trait]
impl Command for DigCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        let args: Vec<&str> = ctx.args.split(' ').collect();
        if args[0].trim().is_empty()
        || args[0].starts_with('?')
        {
            ctx.args = "dig";
            let help = HelpCommand;
            return help.exec(ctx).await;
        }
        let dir = Direction::try_from(args[0]);
        if let Err(_) = dir {
            tell_user!(ctx.writer, "No such direction exists... See <c yellow>'help dir'</c>.\n");
            resume_game!(ctx);
        }

        let room = Room::blank();

        resume_game!(ctx);
    }
}
