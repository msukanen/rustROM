//! Graceful 'shutdown' of the server.

use std::time::Duration;

use async_trait::async_trait;

use crate::{cmd::{Command, CommandCtx}, show_help_if_needed, tell_user, traits::save::DoesSave, util::{Broadcast, comm::{SystemBroadcastType, TellFrom}}, validate_admin};

pub(crate) struct ShutdownCommand;

const MIN_SHUTDOWN_SECONDS: u64 = 15;
const DEFAULT_SHUTDOWN_SECONDS: u64 = 30;
const MAX_SHUTDOWN_SECONDS: u64 = 900;// 15 min

#[async_trait]
impl Command for ShutdownCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        validate_admin!(ctx);

        let delay = if ctx.args.is_empty() {
            DEFAULT_SHUTDOWN_SECONDS
        } else if let Ok(delay) = ctx.args.parse::<u64>() {
            // Clamp to the delay into min..max range to leave breathing
            // room for the shutdown procedures (player saves, etc.)
            // and yet not wait forevermore.
            delay.clamp(MIN_SHUTDOWN_SECONDS, MAX_SHUTDOWN_SECONDS)
        } else {
            show_help_if_needed!(ctx, "shutdown");
            tell_user!(ctx.writer, "Sorry, but I cannot let you do it, Dave. '{}' is not a valid value in seconds…\n", ctx.args);
            return ;
        };

        log::info!("Shutdown sequence initiated: Players notified…");
        let _ = ctx.tx.send(Broadcast::System(SystemBroadcastType::Shutdown {
            message: format!("\n<c red>*** SERVER SHUTDOWN IN {delay} SECONDS ***</c>\n"),
            seconds: delay,
        }));

        log::info!("Saving spatial fabric…");
        if let Err(e) = ctx.world.write().await.save().await {
            log::error!("CRITICAL: World save failed during shutdown: {e:?}");
        }

        tokio::spawn(async move {
            if delay > 10 {
                tokio::time::sleep(Duration::from_secs(delay - 10)).await;
            } else {
                tokio::time::sleep(Duration::from_secs(delay)).await;
            }

            log::warn!("Mistyria is going dark. Terminating…");
            std::process::exit(0);// the final "off" switch…
        });
    }
}
