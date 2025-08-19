use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, show_help, string::styling::RULER_LINE, tell_user, util::{ed::{edit_text, EdResult}, Editor}, validate_builder};

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
            Ok(EdResult::ContentReady { text, verbose, .. }) => {
                let mut g = ctx.player.write().await;
                g.hedit.set_description(&text);
                verbose
            },
            Ok(EdResult::NoChanges(true)) => true,
            Ok(EdResult::HelpRequested) => {
                show_help!(ctx, "hedit-desc");
            },
            _ => false
        };
        
        if verbose {// re-run argless to pretty-print current description.
            let cmd = DescCommand;
            cmd.exec({ctx.args = ""; ctx}).await;
        }
    }
}
