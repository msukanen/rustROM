use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, show_help, show_help_if_needed, tell_user, traits::Identity, util::Broadcast, validate_admin};

pub struct ForceCommand;
#[derive(Debug, Clone)]
pub enum ForceSource {
    Admin { id: String, anonymous: bool },
    System
}

impl Identity for ForceSource {
    fn id<'a>(&'a self) -> &'a str {
        match self {
            Self::Admin { id, .. } => &id,
            Self::System => "«system»",
        }
    }
}

#[async_trait]
impl Command for ForceCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_admin!(ctx);
        show_help_if_needed!(ctx, "force");

        let args = ctx.args.splitn(2, ' ').collect::<Vec<&str>>();
        let from_player = ForceSource::Admin {
            id: ctx.player.read().await.id().to_string(),
            anonymous: {
                ctx.args = args[1];
                args[0] == "-"
            }
        };

        let (to_player, message) = {
            let xs = ctx.args.splitn(2, ' ').collect::<Vec<&str>>();
            if xs.len() < 2 {
                show_help!(ctx, "force");
            }
            (xs[0], xs[1])
        };
        let to_player = match to_player.trim() {
            "self" => {
                tell_user!(ctx.writer, "<c red>ERROR:</c> Cannot target self with <c yellow>'force'</c>.\n\n");
                show_help!(ctx, "force");
            },
            "all" => None,
            _ => Some(to_player.trim().to_string())
        };
        let message = message.trim();

        let force = Broadcast::Force { message: message.into(), to_player, from_player };
        let _ = ctx.tx.send(force);
    }
}
