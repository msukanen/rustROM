//! Who's online and where…

use async_trait::async_trait;

use crate::{cmd::{Command, CommandCtx}, tell_user, traits::{IdentityQuery, mob::IsMob}};
pub struct WhoCommand;

#[async_trait]
impl Command for WhoCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        let w = ctx.world.read().await;
        let mut output = format!("<c yellow>Players currently in Mistyria:</c>\n");
        output.push_str("<c gray>------------------------------</c>\n");
        let mut visibles = 0;

        for p_arc in w.players.values() {
            let p = p_arc.read().await;
            // skip invisibles...
            if p.invis() { continue; }
            
            visibles += 1;

            output.push_str(&format!(
                "  [<c cyan>{:^10}</c>] @ {}\n",
                p.id(),
                p.location
            ));
        }

        output.push_str("<c gray>------------------------------</c>\n");
        output.push_str(&format!("Total players online: {}\n", if ctx.player.read().await.access.is_admin() {
            w.players.len()
        } else {visibles}));
        tell_user!(ctx.writer, "{}", output)
    }
}
