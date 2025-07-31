pub(crate) mod save;
pub(crate) mod access;
use access::Access;
use serde::{Deserialize, Serialize};

use crate::{mob::core::IsMob, player::save::SaveFile};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Player {
    pub(super) access: Access,
    pub(super) save: SaveFile,
}

impl IsMob for Player {
    fn name<'a>(&'a self) -> &'a str {
        self.save.name()
    }
}
