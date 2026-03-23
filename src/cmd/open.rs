//! 'open' command.
//! 
//! 'Open' is used — surprise much? – to open e.g. doors…

use async_trait::async_trait;

use crate::{cmd::{Command, CommandCtx}, show_help_if_needed};

pub struct OpenCommand;

#[async_trait]
impl Command for OpenCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "open");

        
    }
}
