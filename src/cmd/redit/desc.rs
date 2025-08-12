use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, util::clientstate::EditorMode, ClientState};

pub struct DescCommand;

const NO_LORE_OR_ADMIN_ONLY: &str = "Well, unfortunately there is no recorded lore about that particular subject, as far as we know…\n";

#[async_trait]
impl Command for DescCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        tell_user!(ctx.writer, "ROOM-EDIT :: DESC\n");
        resume_game!(ctx);
    }
}
