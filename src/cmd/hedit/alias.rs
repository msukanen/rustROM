use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{help::HelpCommand, Command, CommandCtx}, resume_game, tell_user, tell_user_unk, validate_builder, ClientState};

pub struct AliasCommand;

#[async_trait]
impl Command for AliasCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);

        if ctx.args.is_empty() {
            let g = ctx.player.read().await;
            let g = g.hedit.as_ref().unwrap().lock.read().await;
            tell_user!(ctx.writer, "Alias{}: {:?}\n", if g.aliases.len() > 1 {"es"} else {""}, g.aliases);
            resume_game!(ctx);
        }

        if ctx.args.starts_with('?') {
            let cmd = HelpCommand;
            return cmd.exec({ctx.args = "hedit-alias"; ctx}).await;
        }

        resume_game!(ctx);
    }
}
