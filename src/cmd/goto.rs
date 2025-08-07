use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, do_in_current_room, mob::{core::IsMob, stat::StatValue}, resume_game, tell_command_usage, tell_user, tell_user_unk, util::direction::Direction, ClientState};

pub struct GotoCommand;

/// Translocate player to some other spot in the world.
#[async_trait]
impl Command for GotoCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        if ctx.args.is_empty() {
            tell_command_usage!(ctx,
                "goto",
                "goes to places …",
                format!("{}{}<c green>Usage:</c> goto [DIR]", r#"
<c yellow>'goto'</c> is used to go to e.g. various directions, like:"#, goto_directions())
            );
        }

        let exit: Result<Direction, _> = Direction::try_from(ctx.args);
        if exit.is_err() {
            tell_user!(ctx.writer, "Unknown direction.\n\n{}\n", goto_directions());
            resume_game!(ctx);
        }

        // See if room has corresponding exit...
        do_in_current_room!(ctx, |room|{
            if let Some(destination_room_name) = room.read().await.exits.get(&exit.unwrap()) {
                tell_user!(ctx.writer, "TODO: yes, there is a room '{}' there, but 'goto' is not yet finished…\n", destination_room_name);
            }
        });

        resume_game!(ctx);
    }
}

fn goto_directions() -> String {r#"

    North, East, South, West,
    NorthEast, NorthWest, SouthEast, SouthWest,
    Up, Down.

"#.to_string()
}
