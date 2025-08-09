use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_command_usage, tell_user, util::direction::Direction, world::room::{Room, RoomError}, ClientState};

pub struct HelpCommand;

#[async_trait]
impl Command for HelpCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        resume_game!(ctx);
    }
}
