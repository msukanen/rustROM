use std::fmt::Display;

use async_trait::async_trait;
use crate::{cmd::{ask::AskCommand, Command, CommandCtx}, show_help_if_needed, tell_user, traits::Identity, util::Broadcast};

/// Say, Exclaim, Ask, etc. subtypes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Subtype {
    Say,
    Ask,
    Exclaim,
}

impl Display for Subtype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Ask => "asks",
            Self::Exclaim => "exclaims",
            Self::Say => "says",
        })
    }
}

pub struct SayCommand;

#[async_trait]
impl Command for SayCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        if ctx.args.is_empty() {
            return tell_user!(ctx.writer, "You hear... crickets?\n");
        }
        show_help_if_needed!(ctx, "say");

        // relay questions to Ask…
        if ctx.args.trim().ends_with('?') {
            return AskCommand.exec(ctx).await;
        }

        let subtype = if ctx.args.ends_with('!') {Subtype::Exclaim} else {Subtype::Say};
        let p = ctx.player.read().await;
        let _ = ctx.tx.send(Broadcast::Say {
            room_id: p.location.clone(),
            message: format!("<c blue>[<c cyan>{}</c>]</c> {}: {}", p.id(), subtype, ctx.args.trim()),
            subtype: Some(subtype),
            from_player: p.id().into(),
        });
    }
}
