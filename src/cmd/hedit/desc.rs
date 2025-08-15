use std::{num::{IntErrorKind, NonZeroU32, NonZeroUsize, ParseIntError}, usize};

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, string::styling::RULER_LINE, tell_user, validate_builder, ClientState};

pub struct DescCommand;
pub const MAX_HELP_DESCRIPTION_LINES: usize = 21; // a modest number, sort of fits on a tiny 80x24 terminal thingydoodah. Takes header, title, etc. into account.

#[async_trait]
impl Command for DescCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) -> ClientState {
        validate_builder!(ctx);

        if ctx.args.is_empty() {
            tell_user!(ctx.writer,
                "{}\n{}<c red>// END</c>\n",
                RULER_LINE,
                ctx.player.read().await.hedit.as_ref().unwrap().lock.read().await.description
            );
            resume_game!(ctx);
        }

        let mut args = ctx.args;
        let mut verbose = false;
        if args.starts_with('v') {
            verbose = true;
            args = &args[1..];
        }
        // '+' -- insert as specified line…
        if args.starts_with('+') {
            let args = args[1..].trim_start().splitn(2, ' ').collect::<Vec<&str>>();
            let lno = args[0].parse::<usize>();
            let lno = match lno {
                Ok(lno) => {
                    if lno > MAX_HELP_DESCRIPTION_LINES {
                        tell_user!(ctx.writer,
                            "<c red>Warning!</c> Maximum help entry description length is limited to {} lines.\nCommand cancelled - no changes made.\n",
                            MAX_HELP_DESCRIPTION_LINES);
                        resume_game!(ctx);
                    }
                    lno
                },
                Err(e) => {
                    tell_user!(ctx.writer, "<c red>Error! </c>{:?}\n", e);
                    resume_game!(ctx);
                }
            };
            {
                let mut g = ctx.player.write().await;
                let g = g.hedit.as_mut().unwrap();
                g.dirty = true;
                let mut g = g.lock.write().await;
                g.description = insert_nth_line(&g.description, lno, if args.len() < 2 {""} else {args[1]});
            }
            if verbose {
                let cmd = DescCommand;
                return cmd.exec({ctx.args = ""; ctx}).await;
            }
            resume_game!(ctx);
        }
        // '-' -- remove a line …
        else if args.starts_with('-') {
            let changed = {
            let mut g = ctx.player.write().await;
            let g = g.hedit.as_mut().unwrap();
            let mut h = g.lock.write().await;
            let res = remove_nth_line(&h.description, &args[1..]);
            let mut changed = false;
            match res {
                Ok((dirty, desc)) => {
                    g.dirty = dirty;
                    if g.dirty {
                        changed = true;
                        h.description = desc;
                    } else {
                        tell_user!(ctx.writer, "Nothing to change - not that many lines to begin with.\n");
                    }
                },
                Err(e) => match e.kind() {
                    IntErrorKind::PosOverflow => {tell_user!(ctx.writer, "Well, there's not quite that many lines to begin with …\n");},
                    IntErrorKind::Zero => {tell_user!(ctx.writer, "Err, line numbers generally are counted from 1 (one) and up …\n");},
                    _ => {tell_user!(ctx.writer, "That's not a valid line number, Dave.\n");}
                }
            }
            changed};
            if changed && verbose {
                let cmd = DescCommand;
                return cmd.exec({ctx.args = ""; ctx}).await;
            }
        }
        // '=' -- full replace
        else if args.starts_with('=') {
            {
                let mut g = ctx.player.write().await;
                let g = g.hedit.as_mut().unwrap();
                g.dirty = true;
                let mut h = g.lock.write().await;
                h.description = format!("{}\n", &args[1..]);
            }
            if verbose {
                let cmd = DescCommand;
                return cmd.exec({ctx.args = ""; ctx}).await;
            } else {
                tell_user!(ctx.writer, "OK - description replaced.\n");
            }
        }
        // Append at end if no sub-command specified.
        else {
            {
                let mut g = ctx.player.write().await;
                let g = g.hedit.as_mut().unwrap();
                g.dirty = true;
                let mut g = g.lock.write().await;
                g.description.push_str(&format!("{}\n", args));
            }
            if verbose {
                let cmd = DescCommand;
                return cmd.exec({ctx.args = ""; ctx}).await;
            } else {
                tell_user!(ctx.writer, "OK - text appended.\n");
            }
            resume_game!(ctx);
        }

        resume_game!(ctx);
    }
}

/// Removes nth line from given `text`.
/// 
/// # Arguments
/// - `text`— text to work with.
/// - `lno_str`— line number, as a string representation.
/// 
/// # Returns
/// 1. `(true, `modified-text`)` — if changes were made.
/// 2. `(false, `original-text`)` — if no changes done.
fn remove_nth_line(text: &str, lno_str: &str) -> Result<(bool, String), ParseIntError> {
    let lno: usize = lno_str.parse()?;
    let lno = NonZeroUsize::new(lno).ok_or_else(||"0".parse::<NonZeroU32>().unwrap_err())?;
    let lno: usize = lno.into();

    if text.lines().count() < lno {
        return Ok((false, text.into()));
    }

    Ok((true, text.lines()
        .enumerate()
        // Keep all lines where the index (0-based) is NOT the one we want to remove (1-based)
        .filter(|(i, _)| *i != lno - 1)
        // Discard the index and just keep the line's text
        .map(|(_, line)| line)
        // Collect the remaining lines into a Vec<&str> and join them with newlines
        .collect::<Vec<&str>>()
        .join("\n")))
}

/// Inserts a new line into a string at the nth position (1-based index).
fn insert_nth_line(text: &str, line_num: usize, text_to_insert: &str) -> String {
    if line_num == 0 {
        return text.to_string(); // Or handle as an error
    }

    let mut lines: Vec<&str> = text.lines().collect();
    let index = line_num - 1;

    if index >= lines.len() {
        // If index is out of bounds, just append
        // TODO: append X empty lines if inserted text would be "far" beyond bounds.
        // TODO: see that X isn't stupidly large number though...
        lines.push(text_to_insert);
    } else {
        lines.insert(index, text_to_insert);
    }

    format!("{}\n", lines.join("\n"))
}

#[cfg(test)]
mod desc_tests {
    use crate::cmd::hedit::desc::remove_nth_line;

    #[test]
    fn test_remove_nth_line() {
        let text = "This text has\n3 lines.\nAt least before removal of line #2.";
        let r = remove_nth_line(text, "2");
        if let Ok((true, res)) = r {
            assert_eq!("This text has\nAt least before removal of line #2.", res.as_str());
        } else {
            panic!("No go!");
        }
    }
}
