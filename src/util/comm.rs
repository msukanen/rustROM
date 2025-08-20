use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{cmd::{force::ForceSource, say::Subtype}, player::Player, traits::describe::Identity, world::{room::find_nearby_rooms, SharedWorld}};

/// Various global channel types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Channel {
    /// Channel for questions & answers.
    Qa,
    /// Channel for newbie questions, talk, help, etc.
    Newbie,
    /// OOC noise …
    Ooc,
    /// Builder-only channel.
    Builder,
    /// Admin-only channel.
    Admin,
    /// Event coordination.
    Event,
}

impl Channel {
    /// Check if the player has permission to listen to this channel.
    pub async fn can_listen(&self, player: &Arc<RwLock<Player>>) -> bool {
        let access = player.read().await.access;
        match self {
            Self::Admin   => access.is_admin(),
            Self::Builder => access.is_builder(),
            Self::Event   => access.is_event_host(),
            Self::Newbie  |
            Self::Ooc     |
            Self::Qa      => true
        }
    }

    /// Default channels to listen to.
    pub fn default_listens() -> HashSet<Channel> {
        let mut channels = HashSet::new();
        channels.insert(Channel::Qa);
        channels.insert(Channel::Newbie);
        channels.insert(Channel::Ooc);
        channels
    }

    /// Some channels are "always on", some are opt-in.
    pub fn is_always_on(&self) -> bool {
        match self {
            Self::Admin   |
            Self::Builder |
            Self::Event   => true,
            Self::Newbie  |
            Self::Ooc     |
            Self::Qa      => false,
        }
    }
}

/// Various broadcast types.
#[derive(Clone, Debug)]
pub enum Broadcast {
    Say {
        subtype: Option<Subtype>,
        room_id: String,
        message: String,
        from_player: String,
    },
    Shout {
        room_id: String,
        message: String,
        from_player: String,
    },
    Tell {
        subtype: Option<Subtype>,
        message: String,
        to_player: String,
        from_player: String,
    },
    /// Special system/admin broadcast.
    Force {
        message: String,
        to_player: Option<String>,
        from_player: ForceSource,
    },
    Channel {
        channel: Channel,
        message: String,
        from_player: String,
    }
}

pub(crate) trait MessagePayload {
    fn message<'a>(&'a self) -> String;
    fn from_player(&self) -> String;
}

impl MessagePayload for Broadcast {
    fn from_player(&self) -> String {
        match self {
            Self::Channel { from_player, .. }|
            Self::Shout { from_player , ..}|
            Self::Say { from_player, .. }|
            Self::Tell { from_player , ..} => from_player.clone(),
            Self::Force { from_player, .. } => from_player.id().to_string(),
        }
    }

    fn message<'a>(&'a self) -> String {
        match self {
            Self::Channel { channel, message, from_player } => {
                match channel {
                    Channel::Admin => format!("[ADMIN-CHAT]({}): {}", from_player, message),
                    Channel::Builder => format!("[BUILD-CHAT]({}): {}", from_player, message),
                    Channel::Event => format!("[EVENT-CHAT]({}): {}", from_player, message),
                    Channel::Newbie => format!("[NEW ONES]({}): {}", from_player, message),
                    Channel::Ooc => format!("[OOC]({}): {}", from_player, message),
                    Channel::Qa => format!("[Q&A]({}): {}", from_player, message),
                }
            }
            Self::Force { message, .. }|
            Self::Say { message, .. }|
            Self::Shout { message, .. }|
            Self::Tell { message, .. } => message.clone()
        }
    }
}

#[async_trait]
pub trait IsRecipient {
    async fn is_recipient(&self, player: &Arc<RwLock<Player>>, world: &SharedWorld) -> bool;
}

#[async_trait]
impl IsRecipient for Broadcast {
    async fn is_recipient(&self, player: &Arc<RwLock<Player>>, world: &SharedWorld) -> bool {
        let p = player.read().await;
        if p.id() == self.from_player() { return false ;}

        match self {
            Self::Say { room_id, ..} => p.location == *room_id,
            Self::Shout { room_id, .. } => {
                let nearby = find_nearby_rooms(world, &room_id, 2).await;
                nearby.contains(&p.location)
            },
            Self::Tell { to_player, .. } => p.id() == *to_player,
            Self::Force { to_player, from_player, .. } => {
                if let Some(to) = to_player { *to == p.id() } else {
                    // prevent force from affecting self.
                    from_player.id() != p.id()
                }
            },
            Self::Channel { channel, .. } => channel.can_listen(&player).await && (p.listening_to(channel) || channel.is_always_on()),
        }
    }
}
