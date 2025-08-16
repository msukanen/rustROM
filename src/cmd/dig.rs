use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use crate::{cmd::{help::HelpCommand, Command, CommandCtx}, resume_game, tell_user, traits::Description, util::direction::Direction, validate_builder, world::room::Room, ClientState};

pub struct DigCommand;

#[async_trait]
impl Command for DigCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);

        if let Some((dir, id)) = validate_args(ctx).await {
            if create_and_link_room(ctx, dir, id).await {
                // no-op for now
            }
        }

        resume_game!(ctx);
    }
}

async fn validate_args<'a>(ctx: &mut CommandCtx<'a>) -> Option<(Direction, &'a str)> {
    let args: Vec<&str> = ctx.args.splitn(2, ' ').collect();
    if args.len() < 2 || args[0].starts_with('?') {
        let cmd = HelpCommand;
        cmd.exec({ctx.args = "dig"; ctx}).await;
        return None;
    }

    let dir = match Direction::from_standard_str(args[0]) {
        Ok(dir) => dir,
        Err(_) => {
            tell_user!(ctx.writer, "<c red>Error!</c> '{}' is not a valid direction.\n\nFor valid values …:\n\n", args[0]);
            let cmd = HelpCommand;
            cmd.exec({ctx.args = "dir"; ctx}).await;
            return None;
        }
    };

    let new_room_id = args[1];

    if !new_room_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        tell_user!(ctx.writer, "<c red>Error!</c> Room IDs can only contain <c cyan>letters</c> (a-z), <c cyan>numbers</c> (0-9), <c cyan>hyphens</c> (-), and <c cyan>underscores</c> (_).\n");
        return None;
    }

    let w = ctx.world.read().await;
    let p = ctx.player.read().await;

    if w.rooms.contains_key(new_room_id) {
        tell_user!(ctx.writer,
            "<c red>A room with that ID already exists!</c>\n\
             Please, choose another ID (or use '<c yellow>redit {}</c>' to modify the existing one).\n",
            new_room_id);
        return None;
    }

    if let Some(curr_room) = w.rooms.get(&p.location) {
        if curr_room.read().await.exits.contains_key(&dir) {
            tell_user!(ctx.writer, "That direction is already taken…\n");
            return None;
        }
    }

    Some((dir, new_room_id))
}

async fn create_and_link_room(ctx: &mut CommandCtx<'_>, dir: Direction, id: &str) -> bool {
    let p = ctx.player.read().await;
    let curr_id = p.location.clone();
    let mut room = Room::blank(Some(id));
    room.exits.insert(dir.opposite(), curr_id.clone());
    let lock = Arc::new(RwLock::new(room));
    let mut w = ctx.world.write().await;
    w.rooms.insert(id.into(), lock.clone());

    if let Some(curr_arc) = w.rooms.get(&curr_id) {
        let mut r = curr_arc.write().await;
        r.exits.insert(dir, id.into());
    } else {
        log::error!("Player '{}' was in a non-existent room '{}'", ctx.player.read().await.id(), curr_id);
        return false;
    }

    tell_user!(ctx.writer, format!("Blank room '<c cyan>{}</c>' created.\n", id));
    true
}
/* 
#[async_trait]
impl Command for DigCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        let args: Vec<&str> = ctx.args.split(' ').collect();
        
        if args[0].trim().is_empty()
        || args[0].starts_with('?')
        || args.len() < 2
        {
            ctx.args = "dig";
            let help = HelpCommand;
            return help.exec(ctx).await;
        }

        // Direction to diggy-dig …
        let dir = Direction::try_from(args[0]);
        if let Err(_) = dir {
            tell_user!(ctx.writer, "No such direction exists... See <c yellow>'help dir'</c>.\n");
            resume_game!(ctx);
        }
        let dir = dir.unwrap();
        
        // Player room id …
        let location = ctx.player.read().await.location.clone();
        
        // Requested ID already exists?
        if ctx.world.read().await.rooms.get(args[1]).is_some() {
            tell_user!(ctx.writer,
                "Duplicate room ID '{}'. One with such name already exists.\n\
                Please, choose another or use <c yellow>'redit {}'</c> to modify the existing one.\n",
                args[1], args[1]);
            resume_game!(ctx);
        }
        // Requested direction is already in use?
        else if ctx.world.read().await.rooms.get(&location).unwrap().read().await.exits.contains_key(&dir) {
            tell_user!(ctx.writer, "That direction, {:?}, is already occupied. Please, choose another direction.\n", dir);
            resume_game!(ctx);
        }

        // Create a blank …
        let mut room = Room::blank();
        room.id = args[1].into();
        // reverse-dir to current room.
        room.exits.insert(dir.opposite(), ctx.world.read().await.rooms.get(location.as_str()).unwrap().read().await.id().into());
        let state = ReditState {
            lock: Arc::new(RwLock::new(room)),
            dirty: true,
        };
        // Connect the blank …
        {
            let mut g = ctx.world.write().await;
            g.rooms.get_mut(location.as_str()).unwrap().write().await.exits.insert(dir, args[1].into());
            g.rooms.insert(args[1].into(), state.lock.clone());
        }

        let mut g = ctx.player.write().await;
        g.redit = Some(state);
        tell_user!(ctx.writer, "Blank room '{}' created.\n", g.redit.as_ref().unwrap().lock.read().await.id());

        resume_game!(ctx);
    }
}
 */