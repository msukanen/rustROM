use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::{cmd::{Command, CommandCtx}, tell_user, traits::Description, util::clientstate::EditorMode, validate_builder, world::room::Room, ClientState};

pub mod desc;

pub struct ReditCommand;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReditState {
    pub entry: Room,
    pub dirty: bool,
}

#[async_trait]
impl Command for ReditCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_builder!(ctx);

        if ctx.args.is_empty() && ctx.player.read().await.redit.is_none() {
            return tell_user!(ctx.writer,
                "ROOM-ID missing and no previous REdit session stored.\n\
                Which room you want to edit? In case of the current one, use '<c yellow>redit this</c>'\n");
        }

        let mut g = ctx.player.write().await;
        if g.redit.is_some() {
            let ed = g.redit.as_mut().unwrap();
            if !ctx.args.is_empty() && ed.entry.id() != ctx.args {
                if ed.dirty {
                    return tell_user!(ctx.writer, "<c red>Warning!</c> Unsaved edits - '<c yellow>save</c>' or '<c yellow>abort</c>' first.\n");
                }
            } else {
                let id = ed.entry.id.clone();
                g.push_state(ClientState::Editing { mode: EditorMode::Room });
                return tell_user!(ctx.writer, "Resuming REdit('{}') session.\n", id);
            }
        }

        if let Some(existing_entry) = ctx.world.read().await.rooms.get(ctx.args) {
            g.redit = Some(ReditState {
                entry: existing_entry.read().await.clone(),
                dirty: false
            });
        } else {
            g.redit = Some(ReditState {
                entry: Room::blank(Some(ctx.args)),
                dirty: true
            });
        }

        g.push_state(ClientState::Editing { mode: EditorMode::Room });
    }
}
