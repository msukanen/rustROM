use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, ClientState};

pub struct HelpCommand;

const NO_LORE_OR_ADMIN_ONLY: &str = "Well, unfortunately there is no recorded lore about that particular subject, as far as we know…\n";

#[async_trait]
impl Command for HelpCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if let Some(help_entry) = ctx.world.read().await.help.get(ctx.args) {
            if !help_entry.read().await.admin || ctx.player.read().await.access.is_admin() {
                tell_user!(ctx.writer, help_entry.read().await.to_string());
                resume_game!(ctx);
            }
        }

        tell_user!(ctx.writer, NO_LORE_OR_ADMIN_ONLY);
        resume_game!(ctx);
    }
}
