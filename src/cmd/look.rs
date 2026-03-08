use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, do_in_current_room, item::inventory::Storage, tell_user, traits::Description};

pub struct LookCommand;

#[async_trait]
impl Command for LookCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        look_at_current_room(ctx).await;
    }
}

/// The looking glass… used by e.g. 'look' command, etc.
pub async fn look_at_current_room(ctx: &mut CommandCtx<'_>) {
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
        tell_user!(ctx.writer, "You see... nothing much else than a wall of white text on a dark surface?\n");
    });
}
