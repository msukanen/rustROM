use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use crate::{cmd::{Command, CommandCtx, redit::ReditCommand, translocate::translocate}, show_help, tell_user, traits::IdentityQuery, util::direction::Direction, validate_builder, world::{exit::{Exit, state::ExitState}, room::Room}};

pub struct DigCommand;

#[async_trait]
impl Command for DigCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_builder!(ctx);

        let vr = validate_args(ctx).await;
        if let Some((dir, id)) = vr {
            let id: String = id.into();
            if create_and_link_room(ctx, dir, &id).await {
                let source = ctx.player.read().await.location.clone();
                let _ = translocate(ctx.world, Some(source), id.into(), ctx.player.clone()).await;
                let cmd = ReditCommand;
                cmd.exec({ctx.args = "this"; ctx}).await;
            }
        }
    }
}

/// Validate 'dig' arguments.
async fn validate_args<'a>(ctx: &mut CommandCtx<'a>) -> Option<(Direction, &'a str)> {
    let args: Vec<&str> = ctx.args.splitn(2, ' ').collect();
    if args.len() < 2 || args[0].starts_with('?') {
        show_help!(ctx, "dig"; None);
    }

    let dir = match Direction::try_from_std(args[0]) {
        Ok(dir) => dir,
        Err(_) => {
            tell_user!(ctx.writer, "<c red>Error!</c> '{}' is not a valid direction.\n\n", args[0]);
            show_help!(ctx, "q dir"; None);
        }
    };

    let new_room_id = args[1];

    if !new_room_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        tell_user!(ctx.writer, "<c red>Error!</c> Room IDs can only contain <c cyan>letters</c> (a-z), <c cyan>numbers</c> (0-9), <c cyan>hyphens</c> (-), and <c cyan>underscores</c> (_).\n");
        return None;
    }

    let w = ctx.world.read().await;
    if w.rooms.contains_key(new_room_id) {
        tell_user!(ctx.writer,
            "<c red>A room with that ID already exists!</c>\n\
             Please, choose another ID (or use '<c yellow>redit {}</c>' to modify the existing one).\n",
            new_room_id);
        return None;
    }

    let location = ctx.player.read().await.location.clone();
    if let Some(curr_room) = w.rooms.get(&location) {
        if curr_room.read().await.exits.contains_key(&dir) {
            tell_user!(ctx.writer, "That direction is already taken…\n");
            return None;
        }
    }

    Some((dir, new_room_id))
}

/// Create a new [Room] at `dir`, making a bi-directional way there from current [Room].
/// 
/// # Args
/// - `ctx`…
/// - `dir` which way to make the new [Room].
/// - `id` for the new [Room].
async fn create_and_link_room(ctx: &mut CommandCtx<'_>, dir: Direction, id: &str) -> bool {
    let curr_id = {
        let p = ctx.player.read().await;
        p.location.clone()
    };
    let mut room = Room::blank(Some(id));
    // by default we use None as key_id - install a lock later…
    if let Ok(opp) = dir.opposite() {
        room.exits.insert(opp, Exit { destination: curr_id.clone(), state: ExitState::Open {key_id: None} });
        tell_user!(ctx.writer, "FYI, don't forget to install a lock, if/when needed (or plausible to even have).\n");
    } else {
        tell_user!(ctx.writer, "Source exit '{:?}' doesn't have a clear opposite.\n = you have to craft return direction manually!\n", dir);
    }
    let mut w = ctx.world.write().await;
    let lock;

    if let Some(curr_arc) = w.rooms.get(&curr_id) {
        let mut r = curr_arc.write().await;
        room.parent_id = r.parent_id.clone();
        room.parent = r.parent.clone();
        lock = Arc::new(RwLock::new(room));
        r.exits.insert(dir, Exit { destination: id.into(), state: ExitState::Open {key_id: None} });
        // we'll insert the room into World a bit later below…
    } else {
        log::error!("Player '{}' was in a non-existent room '{}'", ctx.player.read().await.id(), curr_id);
        return false;
    }
    // …pesky borrows made me put this line here instead of the if-block above…
    w.rooms.insert(id.into(), lock.clone());
    log::debug!("Room inserted.");

    tell_user!(ctx.writer, format!("Blank room '<c cyan>{}</c>' created.\n", id));
    true
}
