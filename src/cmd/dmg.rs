use async_trait::async_trait;
use crate::{cmd::{help::HelpCommand, Command, CommandCtx}, mob::{core::IsMob, stat::StatValue}, tell_user, tell_user_unk};

pub struct DmgCommand;

#[async_trait]
impl Command for DmgCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        if !ctx.player.read().await.access.is_admin() {
            return tell_user_unk!(ctx.writer);
        }

        let args: Vec<&str> = ctx.args.split(' ').collect();
        log::info!("ARGL: {} '{}'", args.len(), args[0]);
        if args.len() == 1 {
            ctx.args = "dmg";
            let help = HelpCommand;
            return help.exec(ctx).await;
        } else {
            match args[0] {
                "self" => {
                    if let Ok(n) = args[1].parse::<i32>() {
                        ctx.player.write().await.take_dmg(n as StatValue);
                    }
                },
                "fix" => {

                },
                _ => {
                    log::warn!("TODO: 'dmg [TARGET]' is a stub.");
                    tell_user!(ctx.writer, "TODO: targeting others.\n");
                }
            }
        }
    }
}
