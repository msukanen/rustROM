use async_trait::async_trait;
use crate::{access_ed_entry, cmd::{Command, CommandCtx}, show_help, util::{ed::{edit_text, EdResult}, Editor}, validate_builder};

pub struct DescCommand;

#[async_trait]
impl Command for DescCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_builder!(ctx);

        let res = edit_text(ctx.writer, ctx.args, &access_ed_entry!(ctx, hedit).description).await;
        
        let verbose = match res {
            // Description needs (re)setting only if 'dirty' flag is `true`.
            Ok(EdResult::ContentReady { text, verbose, dirty: true }) => {
                let mut g = ctx.player.write().await;
                g.hedit.set_description(&text);
                verbose
            },
            Ok(EdResult::NoChanges(true)) => true,
            Ok(EdResult::HelpRequested) => {
                show_help!(ctx, "edit-desc");
            },
            _ => false
        };
        
        if verbose {// re-run argless to pretty-print current description.
            let cmd = DescCommand;
            cmd.exec({ctx.args = ""; ctx}).await;
        }
    }
}
