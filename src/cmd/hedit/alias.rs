use async_trait::async_trait;
use crate::{cmd::{help::HelpCommand, Command, CommandCtx}, resume_game, tell_user, traits::save::DoesSave, validate_builder, ClientState};

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

        // Display help entry.
        if ctx.args.starts_with('?') {
            let cmd = HelpCommand;
            return cmd.exec({ctx.args = "hedit-alias"; ctx}).await;
        }

        let parts = ctx.args.split(' ')
                .map(|w| w.trim())
                .filter(|v| v.len() >= 2 && {
                    let c = v.chars().nth(0).unwrap();
                    c == '-' || c == '+'
                })
                .filter(|v| v.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '+'))
                .collect::<Vec<&str>>();
        let mut g = ctx.player.write().await;
        let g = g.hedit.as_mut().unwrap();
        let mut h = g.lock.write().await;
        let orig_aliases = h.aliases.clone();
        let mut req_change = orig_aliases.clone();

        for part in parts {
            if part.starts_with('+') {
                req_change.insert(part[1..].into());
            }
            else if part.starts_with('-') {
                if h.id == &part[1..] {
                    tell_user!(ctx.writer, "<c red>Warning!</c> Help entry's primary ID '{}' cannot be unaliased!\n", h.id);
                } else {
                    req_change.remove(&part[1..]);
                }
            }
            else {
                todo!("Dev alert! Something didn't get tested!");
            }
        }

        let net_add: Vec<_> = req_change.difference(&orig_aliases).cloned().collect();
        let net_rem: Vec<_> = orig_aliases.difference(&req_change).cloned().collect();
        if !net_add.is_empty() || !net_rem.is_empty() {
            g.dirty = true;
            h.aliases = req_change;
        }

        if !net_add.is_empty() {
            tell_user!(ctx.writer, "Added alias{}: {:?}\n", if net_add.len() == 1 {""} else {"es"}, net_add);
        }
        if !net_rem.is_empty() {
            tell_user!(ctx.writer, "Removed alias{}: {:?}\n", if net_rem.len() == 1 {""} else {"es"}, net_rem);
        }
        
        if let Err(_) = h.save().await {
            tell_user!(ctx.writer,
                "Something went wrong (with the file system perhaps)… The error has been logged.\n\
                Admins might get things sorted out, however — no need to be alarmed (too much).");
        }

        resume_game!(ctx);
    }
}
