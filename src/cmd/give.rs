//! Give something to someone else.
use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;

use crate::{cmd::{Command, CommandCtx}, show_help_if_needed};

pub struct GiveCommand;

lazy_static! {
    static ref GIVE_RX: Regex = Regex::new(r#"^\s*(?P<>)"#).unwrap();
}

#[async_trait]
impl Command for GiveCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        show_help_if_needed!(ctx, "give");

        
    }
}
