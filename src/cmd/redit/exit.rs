//! Room exit control.

use async_trait::async_trait;

use crate::{access_ed_entry, cmd::{Command, CommandCtx}, show_help_if_needed, tell_user, util::direction::Direction, validate_builder};

pub struct ExitCommand;

#[async_trait]
impl Command for ExitCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_builder!(ctx);

        if ctx.args.is_empty() {
            let mut output = "<c yellow>-[ EXITS ]-</c>\n".to_string();
            for (dir, target) in &access_ed_entry!(ctx, redit).exits {
                output.push_str(&format!("  <c cyan>{:<10}</c> → {}\n", dir, target));
            }
            tell_user!(ctx.writer, output);
            return;
        }
        show_help_if_needed!(ctx, "redit-exit");

        let parts = ctx.args
            .split(' ')
            .map(|w| w.trim())
            .filter(|v| !v.is_empty())
            .collect::<Vec<&str>>();
        let mut g = ctx.player.write().await;
        let ed = g.redit.as_mut().unwrap();
        let mut req_change = ed.entry.exits.clone();
        let mut i = 0;
        let mut error = false;
        while i < parts.len() {
            let part = parts[i];
            // add new exit (or modify existing one)
            if part.starts_with('+') {
                let dir = &part[1..];
                if let Some(target) = parts.get(i+1) {
                    req_change.insert(Direction::from(dir), target.into());
                    i += 2;
                    continue;
                }

                tell_user!(ctx.writer, "<c red>Error:</c> Direction '{}' needs a target!\n", dir);
                error = true;
            }
            // remove an exit
            else if part.starts_with('-') {
                req_change.remove(&Direction::from(&part[1..]));
            }
            i += 1;
        }

        if error {
            tell_user!(ctx.writer, "<c red>Edit aborted</c> due errorneous input.\n");
            return;
        }

        if req_change != ed.entry.exits {
            ed.entry.exits = req_change;
            ed.dirty = true;
            tell_user!(ctx.writer, "Exits updated.\n");
            drop(g);
            ExitCommand.exec({ctx.args = ""; ctx}).await;
            return;
        }

        tell_user!(ctx.writer, "Twiddling thumbs, pondering exits?\n");
    }
}
