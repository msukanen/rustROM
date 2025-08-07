/// Shorthand for returning from a variety of commands into 'playing'
/// state of existence.
#[macro_export]
macro_rules! resume_game {
    ($ctx:expr) => {
        return ClientState::Playing;
    };
}

/// Do something in current room of residence...
/// 
/// # Arguments
/// - `$ctx`— [CommandCtx]
/// - `|room|`— will hold on to found room.
/// - `$block`— some block of code.
/// - [otherwise] `$otherwise`— execute this block if 'room' is for some reason unavailable.
#[macro_export]
macro_rules! do_in_current_room {
    ($ctx:ident, |$room:ident| {$($block:tt)*} otherwise {$($otherwise:tt)*}) => {
        if let Some(area) = $ctx.world.read().await.areas.get(&$ctx.player.read().await.location.area) {
            if let Some($room) = area.read().await.rooms.get(&$ctx.player.read().await.location.room) {
                $($block)*
            } else {
                //TODO: safe transfer!
                $($otherwise)*
            }
        } else {
            //TODO: safe transfer!
        }
    };

    ($ctx:ident, |$room:ident| {$($block:tt)*}) => {
        if let Some(area) = $ctx.world.read().await.areas.get(&$ctx.player.read().await.location.area) {
            if let Some($room) = area.read().await.rooms.get(&$ctx.player.read().await.location.room) {
                $($block)*
            }
        } else {
            //TODO: safe transfer!
        }
    };
}

#[macro_export]
macro_rules! tell_command_usage {
    ($ctx:ident, $cmd_name:literal, $brief:literal, $usage:expr) => {
        {
            let usage = format!("<c green>COMMAND </c><c yellow>'{}'</c> - {}\n{}\n\n", $cmd_name, $brief, $usage);
            tell_user!($ctx.writer, crate::string::styling::format_color(&usage));
            resume_game!($ctx);
        }
    };
}
