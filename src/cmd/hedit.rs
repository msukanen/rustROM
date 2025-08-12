use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, util::clientstate::EditorMode, ClientState};

pub struct HeditCommand;

const NO_LORE_OR_ADMIN_ONLY: &str = "Well, unfortunately there is no recorded lore about that particular subject, as far as we know…\n";

#[async_trait]
impl Command for HeditCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if match ctx.player.read().await.state() {
            ClientState::Editing { mode } => match mode {
                EditorMode::Help { .. } => false,
                _ => true
            },
            _ => true
        } {
            ctx.player.write().await.push_state(ClientState::Editing { mode: EditorMode::Help { topic: ctx.args.into() } });
        }
        ctx.player.read().await.state()
    }
}
