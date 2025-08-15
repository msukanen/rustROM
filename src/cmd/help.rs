use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, ClientState};

pub struct HelpCommand;

const NO_LORE_OR_ADMIN_ONLY: &str = "Well, unfortunately there is no recorded lore about that particular subject, as far as we know…\n";
pub(crate) const ERROR_SAVING_HELP: &str = "Something went wrong (with the file system perhaps)… The error has been logged.\n\
                                            Admins might get things sorted out, however — no need to be alarmed (too much).\n";
#[async_trait]
impl Command for HelpCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if let Some(help_entry) = ctx.world.read().await.help.get(ctx.args) {
            let (admin_only, builder_only) = {
                let g = help_entry.read().await;
                let a = g.admin;
                let b = g.builder;
                (a,b)
            };
            let (is_admin, is_builder) = {
                let g = ctx.player.read().await;
                let a = g.access.is_admin();
                let b = g.access.is_builder();
                (a,b)
            };
            
            if (!admin_only || is_admin) &&
               (!builder_only || is_builder)
            {
                tell_user!(ctx.writer, help_entry.read().await.to_string());
                resume_game!(ctx);
            }
        }

        tell_user!(ctx.writer, NO_LORE_OR_ADMIN_ONLY);
        resume_game!(ctx);
    }
}
