use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_command_usage, tell_user, util::direction::Direction, world::room::{Room, RoomError}, ClientState};

pub struct DigCommand;

#[async_trait]
impl Command for DigCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        let args: Vec<&str> = ctx.args.split(' ').collect();
        if args[0].trim().is_empty()
        || args[0].starts_with('?')
        {
            tell_command_usage!(ctx,
                "dig",
                "diggy-diggy-hole, dwarven style…",
r#"<c yellow>'dig'</c> command is used to carve out a new <c cyan>[Room]</c> at some given direction.
If the command succeeds, a new <c cyan>[Room]</c> will be created, you will be transported
there, and your normal command interface is switched to that of the builder tools."#,
                "dig [DIR]"
            );
        }
        let dir = Direction::try_from(args[0]);
        if let Err(_) = dir {
            tell_user!(ctx.writer, "No such direction exists... See <c yellow>'help dir'</c>.\n");
            resume_game!(ctx);
        }
        resume_game!(ctx);
    }
}
