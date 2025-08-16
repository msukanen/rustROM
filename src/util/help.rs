use std::{collections::{HashMap, HashSet}, path::PathBuf, str::FromStr, sync::Arc};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use walkdir::WalkDir;

use crate::{traits::{save::{DoesSave, SaveError}, Description}, DATA_PATH};

static HELP_PATH: Lazy<Arc<String>> = Lazy::new(|| Arc::new(format!("{}/help", *DATA_PATH)));

/// Generic help/manual/doc struct.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Help {
    /// Stem name, etc.
    pub id: String,
    /// Free form title of the help entry.
    pub title: String,
    /// Keywords/aliases.
    pub aliases: HashSet<String>,
    pub description: String,
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub builder: bool,
}

impl Description for Help {
    fn description<'a>(&'a self) -> &'a str { &self.description }
    fn id<'a>(&'a self) -> &'a str { &self.id }
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
    /// Load all help files into hashmap, properly aliased too while at it.
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

    /// Generate a brand new shiny help entry.
    pub(crate) fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            title: "".into(),
            aliases: { let mut h = HashSet::new(); h.insert(id.into()); h },
            description: "".into(),
            admin: false,
            builder: false,
        }
    }
}

impl std::fmt::Display for Help {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "---[ <c green>{}</c> ]---\n  -| {}\n\n{}\n",
            self.id().to_uppercase(),
            self.title(),
            self.description()
        )
    }
}

#[async_trait]
impl DoesSave for Help {
    async fn save(&mut self) -> Result<(), SaveError> {
        if self.id().is_empty() { return Err(SaveError::NoIdProvided); }

        let filename = format!("{}/{}.toml", *HELP_PATH, self.id());
        let tmp_filename = format!("{}.tmp", filename);
        let path = PathBuf::from_str(&tmp_filename).unwrap();
        
        let contents = toml::to_string_pretty(&self);
        if let Err(e) = contents {
            log::error!("TOML format error with '{}': {:?}", self.id(), e);
            return Err(e.into());
        }
        // Save the .tmp file first...
        let err = tokio::fs::write(path, contents.unwrap()).await;
        if let Err(e) = err {
            log::error!("File error with '{}': {:?}", self.id(), e);
            return Err(e.into());
        }
        // Copy .tmp over (potentially existing) original...
        if let Err(e) = tokio::fs::copy(&tmp_filename, &filename).await {
            log::error!("FATAL ERROR - cannot copy temporary file '{}' over '{}': {:?}", tmp_filename, filename, e);
            return Err(e.into());
        }
        tokio::fs::remove_file(&tmp_filename).await?;// this *should* succeed, but who knows...
        Ok(())
    }
}
