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
        {
            let area_name = $ctx.player.read().await.location.area.clone();// ← to avoid borrow issues…
            let room_name = $ctx.player.read().await.location.room.clone();// ← to avoid borrow issues…
            if let Some(area) = $ctx.world.read().await.areas.get(&area_name) {
                if let Some($room) = area.read().await.rooms.get(&room_name) {
                    $($block)*
                }
            } else {
                //TODO: safe transfer!
            }
        }
    };
}

#[macro_export]
macro_rules! tell_command_usage {
    ($ctx:ident, $cmd_name:literal, $brief:literal, $usage:expr) => {
        {
            let usage = format!("<c green>COMMAND </c><c yellow>'{}'</c> - {}\n\n{}\n\n", $cmd_name, $brief, $usage);
            tell_user!($ctx.writer, crate::string::styling::format_color(&usage).as_str());
            resume_game!($ctx);
        }
    };
}

/// Check Read-only field.
/// NOTE: this macro has to be called from an async context!
#[macro_export]
macro_rules! check_ro_field {
    ($ctx:expr, $field:expr, $accessor:ident) => {{
        let w = $ctx.world.read().await;
        if let Some(g) = &w.$accessor {
            let desc = format!("<c yellow>--[ <c green>{}</c> ], current value:--</c>\n{}", $field, g);
            tell_user!($ctx.writer, &desc);
        } else {
            tell_user!($ctx.writer, "<c red>'{}' not set</c>. Use: <c yellow>set {} [VALUE]", $field, $field);
        }
    }};
}
