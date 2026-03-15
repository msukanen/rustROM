//! Looking around, looking at, looking into…
use std::fmt::Display;

use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, do_in_current_room, item::inventory::Storage, tell_user, traits::{Description, Identity}};

pub struct LookCommand;

#[derive(Debug, Clone, Copy)]
enum LookSpecifier {
    At,
    In,
    General,
}

impl From<&str> for LookSpecifier {
    fn from(value: &str) -> Self {
        match value {
            "at"|"@" => Self::At,
            "in" => Self::In,
            _ => Self::General
        }
    }
}

impl Display for LookSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::At => "at",
            Self::In => "in",
            Self::General => "around"
        })
    }
}

#[async_trait]
impl Command for LookCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        let input = ctx.args.trim().split(' ').collect::<Vec<&str>>();
        let spec = LookSpecifier::from(input[0]);
        match spec {
            LookSpecifier::At |
            LookSpecifier::In => {
                match input.get(1) {
                    None => tell_user!(ctx.writer, "Look {} what exactly…?\n", spec),

                    // potential reconstructuction of multipart iten name with .join()…
                    Some(_) => look_at_or_into(ctx, spec, &input[1..].join(" ")).await
                }
            },

            _ => {
                let target = input[0].trim();
                if target.len() > 0 {
                    // feed ctx.args directly so that we get multipart item names right…
                    look_at_or_into(ctx, LookSpecifier::At, ctx.args.trim()).await;
                    return;
                }

                look_at_current_room(ctx).await
            }
        };
    }
}

/// Self-admiration at its best!
macro_rules! admire_self {
    ($ctx:expr) => {{
        tell_user!($ctx.writer, "You look at yourself. You look ready for mischief!\n");
        return;
    }};
}

/// Look at/into something.
async fn look_at_or_into(ctx: &mut CommandCtx<'_>, spec: LookSpecifier, target: &str) {
    let target_lc = target.to_lowercase();

    match target_lc.as_str() {
        "me"|"self"|"myself" => admire_self!(ctx),
        _ if target_lc == ctx.player.read().await.id() => admire_self!(ctx),
        _ => ()
    }

    tell_user!(ctx.writer, "Looking {} {} …\n", spec, target);
}

/// The looking glass… used by e.g. 'look' command, etc.
pub(crate) async fn look_at_current_room(ctx: &mut CommandCtx<'_>) {
    do_in_current_room!(ctx, |room| {
        let r = room.read().await;
        let mut desc = format!(
            "<c yellow>{}</c>\n\n{}\n\n",
            r.title(),
            r.description()
        );

        /* ITEMS ON FLOOR */{
            if !r.is_empty() {
                for (hash, _) in r.contents.items() {
                    desc.push_str(&format!("  <c red>//</c> {}\n", hash));
                }
                desc.push_str("\n");
            }
        }

        /* PEOPLE */{
            if !r.players.is_empty() {
                for p in r.players.keys() {
                    desc.push_str(&format!("    <c blue>[<c cyan>{}</c>]</c>\n", p));
                }
                desc.push_str("\n");
            }
        }

        /* EXITS */{
            if !r.exits.is_empty() {
                desc.push_str("<c green>Exits:</c> ");
                let exits: Vec<String> = r.exits.keys().map(|d| format!("{:?}", d).to_lowercase()).collect();
                desc.push_str(&exits.join(", "));
                desc.push_str("\n\n");
            }
        }
        tell_user!(ctx.writer, &desc);
    } otherwise {
        tell_user!(ctx.writer, "You see… nothing much else than a wall of white text on a dark surface?\n");
    });
}
