use std::num::{IntErrorKind, NonZeroU32, NonZeroUsize, ParseIntError};

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::{cmd::{Command, CommandCtx}, resume_game, string::styling::RULER_LINE, tell_user, validate_builder, ClientState};

pub struct DescCommand;

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
        // '+' -- append at end of the existing …
        if args.starts_with('+') {
            {
                let mut g = ctx.player.write().await;
                let g = g.hedit.as_mut().unwrap();
                g.dirty = true;
                let mut g = g.lock.write().await;
                g.description.push_str(&format!("{}\n", &args[1..]));
            }
            if verbose {
                let cmd = DescCommand;
                return cmd.exec({ctx.args = ""; ctx}).await;
            }
            resume_game!(ctx);
        } else if args.starts_with('-') {
            let mut g = ctx.player.write().await;
            let g = g.hedit.as_mut().unwrap();
            let mut h = g.lock.write().await;
            let res = remove_nth_line(&h.description, &args[1..]);
            match res {
                Ok((dirty, desc)) => {
                    g.dirty = dirty;
                    if dirty {
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
        }

        resume_game!(ctx);
    }
}

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
