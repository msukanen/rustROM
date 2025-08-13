use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, tell_user, util::clientstate::EditorMode, validate_builder, ClientState};

pub mod desc;

pub struct ReditCommand;

const NO_LORE_OR_ADMIN_ONLY: &str = "Well, unfortunately there is no recorded lore about that particular subject, as far as we know…\n";

#[async_trait]
impl Command for ReditCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);
        
        if match ctx.player.read().await.state() {
            ClientState::Editing { mode, .. } => match mode {
                EditorMode::Room { .. } => false,
                _ => true
            },
            _ => true
        } {
            ctx.player.write().await.push_state(ClientState::Editing { mode: EditorMode::Room });// TODO
        }
        ctx.player.read().await.state()
    }
}
