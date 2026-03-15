use std::{collections::HashMap, sync::{Arc, OnceLock}};

use async_trait::async_trait;
use tokio::sync::RwLock;
use crate::{cmd::{Command, CommandCtx}, show_help_if_needed, tell_user, util::Help};

pub struct HelpCommand;

const NO_LORE_OR_ADMIN_ONLY: &str = "Well, unfortunately there is no recorded lore about that particular subject, as far as we know…\n";
pub(crate) static HELP_REGISTRY: OnceLock<RwLock<(HashMap<String, Arc<RwLock<Help>>>, HashMap<String, String>)>> = OnceLock::new();

#[macro_export]
macro_rules! help_reg_lock {
    ($mode:ident) => {
        crate::cmd::help::HELP_REGISTRY.get()
            .expect("Help system failed to init?!")
            .$mode().await
    };
}

#[async_trait]
impl Command for HelpCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        let (is_admin, is_builder) = {
            let g = ctx.player.read().await;
            let a = g.access.is_admin();
            let b = g.access.is_builder();
            (a,b)
        };

        let (quick, args) = {
            let args = ctx.args.splitn(2, ' ').collect::<Vec<&str>>();
            let h = help_reg_lock!(read);
            if args[0].trim().is_empty() {
                // With no arguments, 'help' shows full index of entries present.
                let mut topics = Vec::new();
                for entry_lock in h.0.values() {
                    if let Ok(e) = entry_lock.try_read() {
                        if (!e.admin || is_admin) && (!e.builder || is_builder) {
                            topics.push(e.id.clone());
                        }
                    } else {
                        log::debug!("Skipping locked entry in help indexer…");
                    }
                }
                topics.sort();
                return if topics.is_empty() {
                    tell_user!(ctx.writer, "Wow, the knowledge library is truly empty. Not even a librarian present?!\n")
                } else {
                    let mut output = String::from("<c yellow>Available Help Topics:</c>\n");
                    let col_w = 18;
                    let cols = 4; // 18x4 fits nicely in 80-wide legacy terminal…
                    for (i, topic) in topics.iter().enumerate() {
                        if i % cols == 0 {
                            output.push_str("→ ");
                        }
                        output.push_str(&format!("{:<width$}", topic, width = col_w));
                        if (i+1) % cols == 0 {
                            output.push('\n');
                        }
                    }
                    if topics.len() % cols != 0 {
                        output.push('\n');
                    }

                    tell_user!(ctx.writer, output);
                };
            }

            let quick = args[0] == "q";
            if quick && args.len() > 1 {
                (quick, args[1])
            } else {
                (false, {
                    show_help_if_needed!(ctx, "help");
                    ctx.args
                })
            }
        };

        let h = help_reg_lock!(read);
        let help = &h.0;
        let help_aliased = &h.1;
        if let Some(help_entry) = help_aliased.get(args) {
            let help_entry = help.get(help_entry);
            if help_entry.is_none() {
                tell_user!(ctx.writer, NO_LORE_OR_ADMIN_ONLY);
                return;
            }
            let help_entry = help_entry.unwrap();
            let (admin_only, builder_only, desc) = {
                let g = help_entry.read().await;
                let a = g.admin;
                let b = g.builder;
                let d = if quick { g.description.clone() } else { g.to_string() };
                (a,b,d)
            };
            
            if (!admin_only || is_admin) &&
               (!builder_only || is_builder)
            {
                return if quick {
                    tell_user!(ctx.writer, "{}\n", desc);
                } else {
                    tell_user!(ctx.writer, desc);
                };
            }
        } else {
            log::debug!("Requested help entry '{}' doesn't exist?", args);
        }

        tell_user!(ctx.writer, NO_LORE_OR_ADMIN_ONLY);
    }
}
