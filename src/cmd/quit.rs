use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, ClientState};

pub struct QuitCommand;

#[async_trait]
impl Command for QuitCommand {
    async fn exec(&self, _: &mut CommandCtx<'_>) -> ClientState {
        ClientState::Logout
    }
}
