use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, tell_user, traits::Identity, util::comm::Channel, ClientState};

pub struct ChannelsCommand;

#[async_trait]
impl Command for ChannelsCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        let access = ctx.player.read().await.access.clone();
        let chlist = Channel::list().into_iter().filter(|c| c.allows_listen(&access)).collect::<Vec<Channel>>();
        tell_user!(ctx.writer, "CHANNELS:\n");
        for ch in chlist {
            tell_user!(ctx.writer, " * {}\n", ch.id());
        }
    }
}
