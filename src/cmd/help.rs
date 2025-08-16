use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, resume_game, tell_user, ClientState};

pub struct HelpCommand;

const NO_LORE_OR_ADMIN_ONLY: &str = "Well, unfortunately there is no recorded lore about that particular subject, as far as we know…\n";

#[async_trait]
impl Command for HelpCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        let (quick, args) = {
            let args = ctx.args.splitn(2, ' ').collect::<Vec<&str>>();
            let quick = args[0] == "q";
            if quick && args.len() > 1 {
                (quick, args[1])
            } else {
                (false, ctx.args)
            }
        };

        let w = ctx.world.read().await;
        if let Some(help_entry) = w.help_aliased.get(args) {
            let help_entry = w.help.get(help_entry).unwrap();
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
                if quick {
                    tell_user!(ctx.writer, "{}\n", help_entry.read().await.description);
                } else {
                    tell_user!(ctx.writer, help_entry.read().await.to_string());
                }
                resume_game!(ctx);
            }
        }

        tell_user!(ctx.writer, NO_LORE_OR_ADMIN_ONLY);
        resume_game!(ctx);
    }
}
