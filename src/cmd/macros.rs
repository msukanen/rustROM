/// Do something in current room of residence...
/// 
/// # Arguments
/// - `$ctx`— [CommandCtx]
/// - `|room|`— will hold on to found room.
/// - `$block`— some block of code.
/// - [otherwise] `$otherwise`— execute this block if 'room' is for some reason unavailable.
#[macro_export]
macro_rules! do_in_current_room {
    ($ctx:ident, |$room:ident| {$($block:tt)*} otherwise {$($otherwise:tt)*}) => {{
        if let Some($room) = $ctx.world.read().await.rooms.get(&$ctx.player.read().await.location) {
            $($block)*
        } else {
            //TODO: safe transfer!
            $($otherwise)*
        }
    }};

    ($ctx:ident, |$room:ident| {$($block:tt)*}) => {{
        let room_name = $ctx.player.read().await.location.clone();// ← to avoid borrow issues…
        if let Some($room) = $ctx.world.read().await.rooms.get(&room_name) {
            $($block)*
        } else {
            //TODO: safe transfer!
        }
    }};
}

/// Check Read-only field.
/// NOTE: this macro has to be called from an async context!
#[macro_export]
macro_rules! check_ro_field {
    ($ctx:expr, $field:expr, $accessor:ident) => {{
        let w = $ctx.world.read().await;
        if let Some(g) = &w.$accessor {
            let desc = format!("<c red>// BEGIN: <c green>{}</c>:</c>\n{}\n<c red>// END</c>\n", $field, g);
            tell_user!($ctx.writer, &desc);
        } else {
            tell_user!($ctx.writer, "<c red>'{}' not set</c>. Use: <c yellow>set {} [VALUE]", $field, $field);
        }
    }};
}

/// Run given `$cmd` with `'?'` argument, which generally (should) bring up a help entry.
#[macro_export]
macro_rules! rerun_with_help {
    ($ctx:ident, $cmd:ident) => {
        {
            let cmd = $cmd;
            return cmd.exec({$ctx.args = "?"; $ctx}).await;
        }
    };
}

/// Show help `$topic` and return from calling function if args for given
/// `$ctx` were empty or start with `?`.
/// 
/// # Calling
/// 1. `show_help_if_needed(ctx, "topic")`
///     … return with `()`, or
/// 2. `show_help_if_needed(ctx, "topic"; $retval)`
///     to return with `$retval`.
#[macro_export]
macro_rules! show_help_if_needed {
    ($ctx:ident, $topic:expr) => {
        if $ctx.args.is_empty() || $ctx.args.starts_with('?')
        {   crate::show_help!($ctx, $topic); }
    };

    ($ctx:ident, $topic:expr; $retval:expr) => {
        if $ctx.args.is_empty() || $ctx.args.starts_with('?')
        {   crate::show_help!($ctx, $topic; $retval); }
    };
}

/// Show help `$topic` and return from calling function.
/// 
/// # Calling
/// 1. `show_help!(ctx, "topic")`
///     … return with `()`, or
/// 2. `show_help!(ctx, "topic"; $retval)`
///     to return with `$retval`.
#[macro_export]
macro_rules! show_help {
    ($ctx:ident, $topic:expr) => {
        return crate::cmd_exec!($ctx, help, &$topic);
    };

    ($ctx:ident, $topic:expr; $retval:expr) => {{
        crate::cmd_exec!($ctx, help, &$topic);
        return $retval;
    }};
}

#[macro_export]
macro_rules! cmd_exec {
    ($ctx:ident, $cmd:ident, $args:expr) => {
        {paste::paste! {
            let cmd = crate::cmd::[<$cmd:lower>]::[<$cmd:camel Command>];
            cmd.exec({$ctx.args = $args; $ctx}).await
        }}
    };

    ($ctx:ident, $cmd:ident) => {
        {paste::paste! {
            let cmd = crate::cmd::[<$cmd:lower>]::[<$cmd:camel Command>];
            cmd.exec({$ctx.args = ""; $ctx}).await
        }}
    };
}
