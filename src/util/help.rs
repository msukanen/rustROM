use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use walkdir::WalkDir;

use crate::{traits::Description, DATA_PATH};

static HELP_PATH: Lazy<Arc<String>> = Lazy::new(|| Arc::new(format!("{}/help", *DATA_PATH)));

/// Generic help/manual/doc struct.
#[derive(Debug, Deserialize, Serialize)]
pub struct Help {
    /// Stem name, etc.
    name: String,
    /// Free form title of the help entry.
    pub title: String,
    /// Keywords/aliases.
    pub aliases: Vec<String>,
    pub description: String,
    #[serde(default)]
    pub admin: bool,
}

impl Description for Help {
    fn description<'a>(&'a self) -> &'a str { &self.description }
    fn name<'a>(&'a self) -> &'a str { &self.name }
    fn title<'a>(&'a self) -> &'a str { &self.title }
}

#[derive(Debug)]
pub enum HelpError {
    Io(std::io::Error),
    Format(toml::de::Error),
}

impl From<std::io::Error> for HelpError { fn from(value: std::io::Error) -> Self { Self::Io(value) }}
impl From<toml::de::Error> for HelpError { fn from(value: toml::de::Error) -> Self { Self::Format(value) }}

impl Help {
    pub(crate) async fn load_all() -> Result<HashMap<String, Arc<RwLock<Help>>>, HelpError> {
        let path = PathBuf::from_str((*HELP_PATH).as_str()).unwrap();
        let mut helps = HashMap::new();
        tokio::fs::create_dir_all(&path).await?;
        
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("toml")
                {continue;}

            if let Some(_) = path.file_stem().and_then(|s| s.to_str()) {
                let content = tokio::fs::read_to_string(&path).await?;
                let help: Help = toml::from_str(&content)?;
                let help = Arc::new(RwLock::new(help));
                for alias in &help.read().await.aliases {
                    helps.insert(alias.clone(), help.clone());
                }
            }
        }

        Ok(helps)
    }
}

impl std::fmt::Display for Help {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "---[ <c green>{}</c> ]---\n  -| {}\n\n{}\n",
            self.name().to_uppercase(),
            self.title(),
            self.description()
        )
    }
}
