use std::{fmt::Display, net::SocketAddr, path::PathBuf, str::FromStr, sync::Arc};

use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2, PasswordHash, PasswordVerifier};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{mob::{core::IsMob, gender::Gender, stat::{StatType, StatValue}, CombatStat}, player::access::Access, traits::{save::{DoesSave, SaveError}, Description}, util::password::{validate_passwd, PasswordError}, world::WorldEntrance, DATA_PATH};
use crate::string::Sluggable;

static SAVE_PATH: Lazy<Arc<String>> = Lazy::new(|| Arc::new(format!("{}/save", *DATA_PATH)));

#[derive(Debug)]
pub enum LoadError {
    InvalidLogin,
    Io(std::io::Error),
    Format(serde_json::Error),
    NoSuchSave,
}

impl From<std::io::Error> for LoadError {
    fn from(value: std::io::Error) -> Self { Self::Io(value)}
}

impl From<serde_json::Error> for LoadError {
    fn from(value: serde_json::Error) -> Self { Self::Format(value)}
}

static DUMMY_SAVE: Lazy<Arc<Player>> = Lazy::new(|| Arc::new(Player {
        name: "dummy".into(),
        passwd: "$argon2id$v=19$m=19456,t=2,p=1$Cg...$....".into(),
        description: "Dummy!".into(),
        gender: Gender::Indeterminate,
        access: Access::Dummy,
        location: WorldEntrance::default(),
        hp: CombatStat::default(StatType::HP),
        mp: CombatStat::default(StatType::MP),
        in_combat: false
    }));

/// Player data lives here!
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Player {
    name: String,
    description: String,
    passwd: String,// argon2 hash
    gender: Gender,
    pub access: Access,
    pub location: WorldEntrance,
    hp: CombatStat,
    mp: CombatStat,
    #[serde(skip, default)]
    in_combat: bool,
}

impl Player {
    pub fn placeholder() -> Self { Self::new("placeholder") }
    /// Generate a new, blank [SaveFile] skeleton.
    pub fn new<S>(name: S) -> Self
    where S: Display,
    {
        Self {
            name: name.to_string(),
            description: "<nothing remarkable>".into(),
            passwd: "".into(),
            gender: Gender::Indeterminate,
            access: Access::default(),
            location: WorldEntrance::default(),
            hp: CombatStat::default(StatType::HP),
            mp: CombatStat::default(StatType::MP),
            in_combat: false,
        }
    }

    /// Bootstrap saves.
    pub async fn bootstrap() -> Result<(), std::io::Error> {
        log::warn!("Bootstrap - generating saves dir '{}'", *SAVE_PATH);
        tokio::fs::create_dir_all((*SAVE_PATH).as_str()).await?;
        log::info!("Bootstrap(save) OK.");
        Ok(())
    }

    /// Set password.
    /// 
    /// # Arguments
    /// - `plaintext_password`— new password.
    /// 
    /// # Returns
    /// Most likely `Ok`…
    pub async fn set_passwd<S>(&mut self, plaintext_passwd: S) -> Result<(), PasswordError>
    where S: Display,
    {
        validate_passwd(&plaintext_passwd.to_string()).await?;
        let salt = SaltString::generate(&mut OsRng);
        let pw_hash = Argon2::default()
            .hash_password(plaintext_passwd.to_string().as_bytes(), &salt)?
            .to_string();
        self.passwd = pw_hash;
        Ok(())
    }

    /// Verify given password vs stored password.
    /// 
    /// # Arguments
    /// - `plaintext_passwd`— some passwordlike thing.
    pub fn verify_passwd<S>(&self, plaintext_passwd: S) -> bool
    where S: Display,
    {
        if self.passwd.is_empty() {
            return false;
        }

        // parse stored hash
        let parsed_hash = match PasswordHash::new(&self.passwd) {
            Ok(hash) => hash,
            Err(_) => return false,
        };

        Argon2::default()
            .verify_password(plaintext_passwd.to_string().as_bytes(), &parsed_hash)
            .is_ok()
    }

    /// Load a save.
    /// 
    /// # Arguments
    /// - `name`— name of character to load.
    /// - `plaintext_passwd`— password.
    /// - `_addr`— `IP:port` of incoming connection.
    ///            Used *exclusively* in non-release modes *and* only with '`localtest`' feature switched on.
    pub async fn load(name: &str, plaintext_passwd: &str, _addr: &SocketAddr) -> Result<Player, LoadError> {
        let filename = format!("{}/{}.save", *SAVE_PATH, name.slugify());
        let path = PathBuf::from_str(&filename).unwrap();
        let save = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => {
                log::warn!("Attempt to load non-existent save '{}' by '{}'…", filename, name);
                let _ = DUMMY_SAVE.verify_passwd(plaintext_passwd);
                return Err(LoadError::NoSuchSave);
            }
        };
        let save: Player = serde_json::from_str(&save)?;
        #[cfg(all(debug_assertions, feature = "localtest"))]
        {   log::debug!("ADDR: {}", _addr.to_string());
            if _addr.to_string().split(":").nth(0).eq(&Some("127.0.0.1")) {
                log::warn!("Local test - bypassing password verification.");
                return Ok(save);
            }
        }
        if save.verify_passwd(plaintext_passwd) {
            Ok(save)
        } else {
            log::warn!("Password failure for user '{}'", name);
            Err(LoadError::InvalidLogin)
        }
    }
    
    /// Set access mode.
    /// 
    /// # Arguments
    /// - `access`— new [Access] specs.
    pub fn set_access(&mut self, access: Access) {
        self.access = access
    }
}

#[async_trait]
impl DoesSave for Player {
    /// Save!
    /// 
    /// # Returns
    /// Success?
    async fn save(&self) -> Result<(), SaveError> {
        let filename = format!("{}/{}.save", *SAVE_PATH, self.name.slugify());
        let path = PathBuf::from_str(&filename).unwrap();
        let file = std::fs::File::create(path)?;
        let _ = serde_json::to_writer(file, &self)?;
        log::info!("Saved '{}'.", filename);
        Ok(())
    }
}

impl IsMob for Player {
    fn prompt<'a>(&'a self) -> String {
        log::debug!("Prompt queried...");
        format!("[hp ({}|{})]#> ", self.hp().current(), self.mp().current())
    }
    fn hp<'a>(&'a self) -> &'a CombatStat { &self.hp }
    fn mp<'a>(&'a self) -> &'a CombatStat { &self.mp }
    fn take_dmg<'a>(&'a mut self, percentage: StatValue) -> bool {// TODO: return alive/dead/etc.
        self.hp -= percentage;
        return false;
    }
}

impl Description for Player {
    fn name<'a>(&'a self) -> &'a str { &self.name }
    fn description<'a>(&'a self) -> &'a str { &self.description }
    /// For [Player], 'title' is the same as their name.
    fn title<'a>(&'a self) -> &'a str { &self.name }
}

#[cfg(test)]
mod savefile_tests {
    use super::*;

    const OK_PASSWORD: &str = "new passw0rd, A very intricate thing";
    const FAIL_PASSWD: &str = "badpass";
    const FAKE_ADDR: &str = "1.1.1.1:1234";

    #[tokio::test]
    async fn create_new_savefile() {
        let _ = env_logger::try_init();
        let mut s = Player::new("TestSaveThing");
        let r = s.set_passwd(OK_PASSWORD).await;
        if let Err(e) = &r {
            log::error!("PWD: {:?}", e);
        }
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn save_savefile() {
        let _ = env_logger::try_init();
        let mut savefile = (*DUMMY_SAVE.as_ref()).clone();
        let _ = savefile.set_passwd(OK_PASSWORD).await;
        let save_content = savefile.save().await;
        assert!(save_content.is_ok());
    }

    #[tokio::test]
    async fn load_savefile() {
        let _ = env_logger::try_init();
        let addr = SocketAddr::from_str(FAKE_ADDR).unwrap();
        let savefile = Player::load("dummy", OK_PASSWORD, &addr).await;
        if let Err(e) = &savefile {
            log::error!("SAV: {:?}", e);
        }
        assert!(savefile.is_ok());
    }

    #[tokio::test]
    async fn load_savefile_wrong_pwd() {
        let _ = env_logger::try_init();
        let addr = SocketAddr::from_str(FAKE_ADDR).unwrap();
        let savefile = Player::load("dummy", FAIL_PASSWD, &addr).await;
        assert!(savefile.is_err());
    }
}
