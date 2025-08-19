use std::{collections::{HashMap, HashSet}, fmt::Display, path::PathBuf, str::FromStr, sync::Arc};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use walkdir::WalkDir;

use crate::{traits::{save::{DoesSave, SaveError}, Description}, util::{Editor, GithubContent}, DATA_PATH};

static HELP_PATH: Lazy<Arc<String>> = Lazy::new(|| Arc::new(format!("{}/help", *DATA_PATH)));
static GITHUB_HELP_REPO: &str = "https://api.github.com/repos/msukanen/rustROM-help/contents";

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

impl std::error::Error for HelpError {}
impl From<std::io::Error> for HelpError { fn from(value: std::io::Error) -> Self { Self::Io(value) }}
impl From<toml::de::Error> for HelpError { fn from(value: toml::de::Error) -> Self { Self::Format(value) }}
impl Display for HelpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Format(e) => write!(f, "TOML format error: {:?}", e),
            Self::Io(e) => write!(f, "I/O error: {:?}", e),
        }
    }
}

impl Help {
    /// Load all help files into hashmap, properly aliased too while at it.
    pub(crate) async fn load_all() -> Result<(HashMap<String, Arc<RwLock<Help>>>, HashMap<String, String>), HelpError>
    {
        let path = PathBuf::from_str((*HELP_PATH).as_str()).unwrap();
        let mut helps = HashMap::new();
        let mut aliases = HashMap::new();
        
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("toml")
                {continue;}

            if let Some(_) = path.file_stem().and_then(|s| s.to_str()) {
                let content = tokio::fs::read_to_string(&path).await?;
                if let Ok(help) = toml::from_str::<Help>(&content) {
                    let help = Arc::new(RwLock::new(help));
                    let primary_id = help.read().await.id.clone();
                    helps.insert(primary_id.clone(), help.clone());
                    for alias in &help.read().await.aliases {
                        aliases.insert(alias.clone(), primary_id.clone());
                    }
                }
            }
        }

        Ok((helps, aliases))
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

    /// Fetch the default help files from a GitHub repo.
    pub(crate) async fn bootstrap(url: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Fetching default help files from GitHub…");

        let client = reqwest::Client::builder().user_agent("RustROM MUD").build()?;
        let repo_url = url.unwrap_or(GITHUB_HELP_REPO.into()).clone();
        
        // get list of files...
        let res = client.get(repo_url).send().await?;
        #[cfg(feature = "ittest")]{
            log::debug!("response {:?}", res);
        }
        let res = res.json::<Vec<GithubContent>>().await?;

        for file in res {
            if file.name.ends_with(".toml") {
                let filepath = format!("{}/{}", *HELP_PATH, file.name);
                let download_url = file.download_url.unwrap();
                match tokio::fs::try_exists(&filepath).await {
                    Ok(true) => {
                        log::info!("Skipping download of '{}'. Corresponding entry '{}' already exists.", download_url, filepath);
                        continue;
                    }
                    _ => {}
                }

                log::info!("  → downloading {}…", file.name);
                let content = client.get(&download_url).send().await?.text().await?;
                #[cfg(feature = "ittest")]{
                    log::debug!("{}", content);
                }
                let help = toml::from_str::<Help>(&content);
                if let Ok(mut help) = help {
                    help.save().await?;
                    log::info!("  ✓ help file '{}' from '{}' stored.", filepath, download_url);
                } else {
                    log::info!("  ✗ file '{}' was not recognized as a help entry. Skipping.", download_url);
                }
            }
        }

        log::debug!("Help files downloaded successfully.");
        Ok(())
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

        // Create help directory if such does not exist...
        let _ = tokio::fs::create_dir_all((*HELP_PATH).to_string()).await?;

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

impl Editor for Help {
    fn set_description(&mut self, desc: &str) {
        self.description = desc.into();
    }
}

#[cfg(test)]
mod help_tests {
    use crate::DATA;
    use super::Help;

    #[tokio::test]
    async fn mock_github_fetch() {
        let _ = env_logger::try_init();
        let _ = DATA.set("./data".into());
        let err = Help::bootstrap(None).await;
        if let Err(e) = err {
            log::error!("ERR {}", e);
        }
    }
}
