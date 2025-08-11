use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, ClientState};

pub struct HelpCommand;

const NO_LORE_OR_ADMIN_ONLY: &str = "Well, unfortunately there is no recorded lore about that particular subject, as far as we know…\n";

#[async_trait]
impl Command for HelpCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
    }
}
