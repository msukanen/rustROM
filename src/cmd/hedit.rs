use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use crate::{cmd::{help::HelpCommand, Command, CommandCtx}, resume_game, tell_user, traits::Description, util::{clientstate::EditorMode, Help}, validate_builder, ClientState};

pub(crate) mod desc;
pub(crate) mod data;
pub(crate) mod save;
pub(crate) mod title;
pub(crate) mod alias;
pub(crate) mod admin;
pub(crate) mod builder;

pub struct HeditCommand;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HeditState {
    pub entry: Help,
    #[serde(skip, default)]
    pub original: Option<Arc<RwLock<Help>>>,
    pub dirty: bool,
}

#[async_trait]
impl Command for HeditCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);

        if ctx.args.is_empty() && ctx.player.read().await.hedit.is_none() {
            tell_user!(ctx.writer, "Which help topic you'd like to edit/create?\n");
            resume_game!(ctx);
        }

        if ctx.args.starts_with('?') {
            let cmd = HelpCommand;
            return cmd.exec({ctx.args = "hedit-internal-commands"; ctx}).await;
        }

        let mut pg = ctx.player.write().await;
        if pg.hedit.is_some() {
            let ed = pg.hedit.as_mut().unwrap();
            if !ctx.args.is_empty() && ed.entry.id() != ctx.args {
                if ed.dirty {
                    tell_user!(ctx.writer, "<c red>Warning!</c> Unsaved edits - '<c yellow>save</c>' or '<c yellow>abort</c>' first.\n");
                    resume_game!(ctx);
                }
            } else {
                tell_user!(ctx.writer, "Resuming edit session.\n");
                pg.push_state(ClientState::Editing { mode: EditorMode::Help });
                resume_game!(ctx);
            }
        }

        if let Some(existing_entry) = ctx.world.read().await.help.get(ctx.args) {
            // Make a working copy of an existing entry.
            pg.hedit = Some(HeditState {
                entry: existing_entry.read().await.clone(),
                original: Some(existing_entry.clone()),
                dirty: false
            });
        } else {
            pg.hedit = Some(HeditState {
                entry: Help::new(ctx.args),
                original: None,
                dirty: true
            });
        }

        pg.push_state(ClientState::Editing { mode: EditorMode::Help });
        pg.state()
    }
}
