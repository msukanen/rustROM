use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, show_help, tell_user, traits::Identity, util::comm::Channel};

pub struct ChannelsCommand;

#[async_trait]
impl Command for ChannelsCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        let access = ctx.player.read().await.access.clone();
        let chlist = Channel::list().into_iter().filter(|c| c.allows_listen(&access)).collect::<Vec<Channel>>();
        let mut out: Vec<String> = vec!["<c green>CHANNELS:</c>".into()];
        for ch in chlist {
            out.push(format!(" <c blue>*</c> {}", ch.id()));
        }
        out.push("\n".into());
        tell_user!(ctx.writer, "{}", out.join("\n"));
        show_help!(ctx, "q channels-opt");
    }
}
