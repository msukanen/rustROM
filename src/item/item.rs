use serde::{Deserialize, Serialize};

use crate::{item::Container, traits::{Description, Identity, Owned}};

pub enum ItemError {
    NoItemSpace(Item),
    NoContainerSpace(Container),
    TooLarge(Container),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Item {
    id: String,
    title: String,
    description: String,
    owner: String,
}

impl Identity for Item {
    fn id<'a>(&'a self) -> &'a str { &self.id }
}

impl Description for Item {
    fn description<'a>(&'a self) -> &'a str { &self.description }
    fn title<'a>(&'a self) -> &'a str { &self.title }
}

impl Owned for Item {
    fn owner(&self) -> &str {
        &self.owner
    }
}

impl Item {
    #[cfg(test)]
    pub fn blank() -> Self {
        use uuid::Uuid;

        Self {
            id: format!("test-item-{}", Uuid::new_v4()),
            title: "test item".into(),
            description: "a very testful item this is, yes".into(),
            owner: "test".into()
        }
    }
}