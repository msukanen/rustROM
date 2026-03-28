//! Looking around, looking at, looking into…
use std::{collections::HashMap, fmt::Display};

use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx, hedit::title}, do_in_current_room, item::inventory::Storage, tell_user, traits::{Description, IdentityQuery}};

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
        _ if matches!(spec, LookSpecifier::At) => do_in_current_room!(ctx, |room| {
            let lock = room.read().await;
            // present players first…
            for p in lock.players.values() {
                if let Some(other) = p.upgrade() {
                    let other = other.read().await;
                    if other.id().contains(&target_lc) {
                        tell_user!(ctx.writer, "Looks like '{}' to you…\n", other.id());
                    }
                }
            }
            // items & other stuff…
            for i in lock.contents.items().values() {
                if i.id().contains(&target_lc) {
                    tell_user!(ctx.writer, "You see… '{}', clearly.\n", i.id());
                }
            }
            // [Player]'s own stuff…
            let lock = ctx.player.read().await;
            for i in lock.inventory.items().values() {
                if i.id().contains(&target_lc) {
                    tell_user!(ctx.writer, "In your inventory you notice… '{}', apparently.\n", i.id());
                }
            }
        }),
        _ => ()
    }
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
                let mut counts = HashMap::new();
                // let's try avoid scroll of doom a bit…
                for item in r.contents.items().values() {
                    *counts.entry(item.title()).or_insert(0) += 1;
                }

                for (title, count) in counts {
                    if count > 1 {
                        desc.push_str(&format!("  <c red>//</c> {title} ({count})\n"));
                    } else {
                        desc.push_str(&format!("  <c red>//</c> {title}\n"));
                    }
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

#[cfg(test)]
mod cmd_look_tests {
    use std::sync::Arc;
    use tokio::{io::{AsyncBufReadExt, AsyncReadExt, BufReader, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::{broadcast, RwLock}};
    use crate::{async_client_for_tests, async_server_for_tests, item::{Item, inventory::{Container, ContainerType, Content}, key::Key}, player::Player, player_and_listener_for_tests, string::ansi::AntiAnsi, util::{Broadcast, ClientState, direction::Direction}, world::{World, area::Area, exit::*, room::Room}, world_for_tests};
    use super::*;

    #[tokio::test]
    async fn at_single_item() {
        let _ = env_logger::try_init();
        log::info!("Preparing the stage …");
        let w = world_for_tests!();
        let (p, listener, addr, tx) = player_and_listener_for_tests!();
        // put some item into room
        // give player the right key...
        {
            let w = w.read().await;
            let r = w.rooms.get("void").unwrap();
            let mut lock = r.write().await;
            let item = Item::Key(Key::new("abloy-key-2", false));
            lock.try_insert(item).unwrap();// we trust the system… *crosses fingers*
        }

        let client_task = async_client_for_tests!(addr, "look", "look at me", "look at abloy");
        let server_task = async_server_for_tests!(w, listener, tx, addr, p, 3);

        // wait for the client task to finish and get the output…
        let (_, client_out) = tokio::join!(server_task, client_task);
        let output_string = client_out.unwrap();
        let output_string = output_string.strip_ansi();

        // assert that the output contains the description of BOTH rooms.
        assert!(output_string.contains("[ani]"));
        assert!(output_string.contains("yourself"));
        assert!(output_string.contains("abloy"));
    }
}