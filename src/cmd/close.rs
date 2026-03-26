//! Close something or other…
//! 
//! 'close' closes a door or such.

use async_trait::async_trait;

use crate::{cmd::{Command, CommandCtx}, do_in_current_room, equalize_opposite_exit_state, show_help_if_needed, string::rx::WHAT_WITH_ARG_RX, tell_user, traits::Identity, util::direction::Direction, world::exit::state::{ExitState, KEY_THAT_IS_NOT_A_KEY}};

pub struct CloseCommand;

#[async_trait]
impl Command for CloseCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "close");

        let mut try_lock = false;
        let (what, with) = if let Some(caps) = WHAT_WITH_ARG_RX.captures(ctx.args) {
            try_lock = true;
            (
                caps.name("what").unwrap().as_str(),
                caps.name("with").unwrap().as_str()
            )
        } else {
            (ctx.args, KEY_THAT_IS_NOT_A_KEY)
        };

        do_in_current_room!(ctx, |room| {
            let mut r = room.write().await;
            let dir = Direction::from(what);
            let r_id = r.id().to_string();
            if let Some(exit) = r.exits.get_mut(&dir) {
                if exit.is_closed() && !try_lock {
                    tell_user!(ctx.writer, "It's already closed.\n");
                    return ;
                }

                if !exit.can_close() {
                    tell_user!(ctx.writer, "Well, honestly — you have no faintest clue how to close that…\n");
                    return ;
                }

                if !exit.state.close() {
                    tell_user!(ctx.writer, "It's not closing. Is it jammed?\n");
                    return ;
                }

                if try_lock {
                    if let Ok(true) = exit.state.lock_with(&with) {
                        tell_user!(ctx.writer, "You close and lock the entrance to '{}'.\n", exit.destination);
                    } else {
                        tell_user!(ctx.writer, "You lack the right key to lock this entrace, but you close it nonetheless.\n");
                    }
                } else {
                    tell_user!(ctx.writer, "You close the way to '{}'.\n", exit.destination);
                }

                equalize_opposite_exit_state!(ctx, r_id, exit);
            } else {
                tell_user!(ctx.writer, "In theory, closing '{}' might work… if it was here, but it isn't.\n", dir);
            }
        });
    }
}
