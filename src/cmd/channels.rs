use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, ClientState};

pub struct ChannelsCommand;

#[async_trait]
impl Command for ChannelsCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        return ;
    }
}
