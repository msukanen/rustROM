use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, tell_user_unk, util::clientstate::EditorMode, ClientState};

pub struct TitleCommand;

#[async_trait]
impl Command for TitleCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        resume_game!(ctx);
    }
}
