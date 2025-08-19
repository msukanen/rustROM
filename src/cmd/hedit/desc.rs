use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, show_help, string::styling::RULER_LINE, tell_user, util::ed::{edit_text, EdResult}, validate_builder};

pub struct DescCommand;

#[async_trait]
impl Command for DescCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_builder!(ctx);

        if ctx.args.is_empty() {
            return tell_user!(ctx.writer,
                "{}\n{}<c red>// END</c>\n",
                RULER_LINE,
                ctx.player.read().await.hedit.as_ref().unwrap().entry.description
            );
        }

        let res = edit_text(ctx.writer, ctx.args, &ctx.player.read().await.hedit.as_ref().unwrap().entry.description).await;
        let verbose = match res {
            Ok(EdResult::ContentReady { text, dirty, verbose }) => {
                let mut g = ctx.player.write().await;
                let ed = g.hedit.as_mut().unwrap();
                ed.dirty = dirty;
                ed.entry.description = text;
                log::debug!("Edited:\n{}", ed.entry.description);
                verbose
            },
            Ok(EdResult::NoChanges(true)) => true,
            Ok(EdResult::HelpRequested) => {
                show_help!(ctx, "hedit-desc");
            },
            _ => false
        };
        
        if verbose {
            let cmd = DescCommand;
            cmd.exec({ctx.args = ""; ctx}).await;
        }
    }
}
