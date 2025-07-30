pub(crate) mod save;
pub(crate) mod access;
use access::Access;

use crate::{mob::core::IsMob, player::save::SaveFile};

pub(crate) struct NewMobPlayer {
    access: Access,
    save: Option<SaveFile>,
}

#[derive(Debug)]
pub(crate) struct Player {
    pub(super) access: Access,
    pub(super) save: SaveFile,
}

impl IsMob for Player {
    fn name<'a>(&'a self) -> &'a str {
        self.save.name()
    }
}

impl NewMobPlayer {
    pub fn promote_to_player(self) -> Player {
        Player {
            access: self.access,
            save: self.save.unwrap()
        }
    }
}
