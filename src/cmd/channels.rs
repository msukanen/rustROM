use async_trait::async_trait;
use crate::{cmd::{Command, CommandCtx}, show_help, string::unicode::{FAILMARK, CHECKMARK}, tell_user, traits::Identity, util::comm::Channel};

pub struct ChannelsCommand;

#[async_trait]
impl Command for ChannelsCommand {
    async fn exec(&self, ctx: &mut CommandCtx<'_>) {
        let access = ctx.player.read().await.access.clone();
        let chlist = Channel::list().into_iter().filter(|c| c.allows_listen(&access)).collect::<Vec<Channel>>();
        
        if ctx.args.starts_with('?') { show_help!(ctx, "channels");}
        
        // List channels with empty args.
        if ctx.args.is_empty() {
            let mut out: Vec<String> = vec!["<c green>CHANNELS:</c>".into()];
            for ch in chlist {
                out.push(format!(" <c gray>*</c> {}", ch.id()));
            }
            out.push("\n".into());
            tell_user!(ctx.writer, "{}", out.join("\n"));
            show_help!(ctx, "q channels-opt");
        }

        let args = ctx.args.splitn(2, ' ').collect::<Vec<&str>>();
        if args.len() < 2 { show_help!(ctx, "channels-opt");}

        let optin = match *args.get(0).unwrap() {
            "in" => true,
            "out" => false,
            _ => { show_help!(ctx, "q channels-opt"); }
        };
        
        // Attempt to find out which channel the user means actually...
        let ch = match Channel::try_from(*args.get(1).unwrap()) {
            Ok(ch) => ch,
            Err(_) => {
                tell_user!(ctx.writer, "<c red>Error:</c> no such channel. See channels list:\n\n");
                return self.exec({ctx.args = ""; ctx}).await;
            }
        };
        
        // Opt-out?
        if !optin {
            if ch.is_always_on() || access.is_admin() {
                tell_user!(ctx.writer, "{FAILMARK} channel '{}' is always on and cannot be opted out from.\n", ch.id());
            } else {
                ctx.player.write().await.listening_to_optout(&ch);
                tell_user!(ctx.writer, "{CHECKMARK} opt-out from '{}' successful.\n", ch.id());
            }
            return ;
        }
        
        if ctx.player.write().await.listening_to_optin(&ch) {
            tell_user!(ctx.writer, "{CHECKMARK} opt-in to '{}' successful.\n", ch.id())
        } else {
            tell_user!(ctx.writer, "{FAILMARK} opt-in to '{}' didn't work out too well...\n", ch.id())
        }
    }
}
