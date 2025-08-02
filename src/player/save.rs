use std::{fmt::Display, path::PathBuf, str::FromStr, sync::Arc};

use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2, PasswordHash, PasswordVerifier};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::{mob::gender::Gender, player::access::Access, traits::save::DoesSave, DATA_PATH};
use crate::string::Sluggable;

pub(crate) static SAVE_PATH: Lazy<Arc<String>> = Lazy::new(|| Arc::new(format!("{}/save", *DATA_PATH)));

#[derive(Debug)]
pub(crate) enum PasswordError {
    TooShort,
    TooSimple,
    NoLowercase, NoUppercase, NoDigit, NoSpecial,
    Argon2Failure(argon2::password_hash::Error),
    NetworkFailure(reqwest::Error),
    HIBPPwned,
}

const MIN_PASSWD_LENGTH: usize = 8;

impl From<argon2::password_hash::Error> for PasswordError {
    fn from(value: argon2::password_hash::Error) -> Self { Self::Argon2Failure(value)}
}

impl From<reqwest::Error> for PasswordError {
    fn from(value: reqwest::Error) -> Self { Self::NetworkFailure(value)}
}

#[derive(Debug)]
pub(crate) enum LoadError {
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

#[derive(Debug)]
pub(crate) enum SaveError {
    PassWordNotSet,
    Io(std::io::Error),
    Format(serde_json::Error),
}

impl From<std::io::Error> for SaveError {
    fn from(value: std::io::Error) -> Self { Self::Io(value)}
}

impl From<serde_json::Error> for SaveError {
    fn from(value: serde_json::Error) -> Self { Self::Format(value)}
}

static DUMMY_SAVE: Lazy<Arc<Player>> = Lazy::new(|| Arc::new(Player {
        name: "dummy".into(),
        passwd: "$argon2id$v=19$m=19456,t=2,p=1$Cg...$....".into(),
        gender: Gender::Indeterminate,
        access: Access::Dummy
    }));

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct Player {
    name: String,
    passwd: String,// hashed stuff...
    gender: Gender,
    access: Access,
}

impl Player {
    /// Generate a new, blank [SaveFile] skeleton.
    pub fn new<S>(name: S) -> Self
    where S: Display,
    {
        Self {
            name: name.to_string(),
            passwd: "".to_string(),
            gender: Gender::Indeterminate,
            access: Access::default(),
        }
    }

    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }

    /// Check pwd pwnage status via HIBP.
    /// 
    /// # Arguments
    /// - `plaintext_password`— password (or alike) to check.
    /// 
    /// # Returns
    /// Either `Ok(())` or e.g. `Err(`[PasswordError::HIBPPwned]`)`.
    async fn is_passwd_pwned(plaintext_passwd: &str) -> Result<(), PasswordError> {
        // Hash the pwd:
        let mut hasher = Sha1::new();
        hasher.update(plaintext_passwd);
        let hash_bytes = hasher.finalize();
        let hash_string = format!("{:X}", hash_bytes);// hex string

        // FYI: HIBP service wants only first 5 bytes of the hash.
        let (prefix, suffix) = hash_string.split_at(5);

        // "Who you gonna call?" - "HIBP…!"
        let url = format!("https://api.pwnedpasswords.com/range/{}", prefix);
        let response = reqwest::Client::new().get(&url).send().await?.text().await?;

        // Check for pwns …
        for line in response.lines() {
            if let Some((pwned_suffix, _)) = line.split_once(':') {
                if pwned_suffix == suffix {
                    return Err(PasswordError::HIBPPwned);
                }
            }
        }
        Ok(())
    }

    /// Validate a password against a set of complexity rules.
    /// 
    /// # Arguments
    /// - `plaintext_password`— password (or alike) to check.
    /// 
    /// # Returns
    /// Either `Ok(())` or the first `Err(`[PasswordError]`)` that matches.
    async fn validate_passwd(plaintext_passwd: &str) -> Result<(), PasswordError>
    {
        if plaintext_passwd.len() < MIN_PASSWD_LENGTH { return Err(PasswordError::TooShort);}
        if !plaintext_passwd.chars().any(|c| c.is_ascii_lowercase()) {return Err(PasswordError::NoLowercase);}
        if !plaintext_passwd.chars().any(|c| c.is_ascii_uppercase()) {return Err(PasswordError::NoUppercase);}
        if !plaintext_passwd.chars().any(|c| c.is_ascii_digit()) {return Err(PasswordError::NoDigit);}
        if !plaintext_passwd.chars().any(|c| c.is_alphanumeric()) {return Err(PasswordError::NoSpecial);}
        Player::is_passwd_pwned(plaintext_passwd).await
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
        Player::validate_passwd(&plaintext_passwd.to_string()).await?;
        let salt = SaltString::generate(&mut OsRng);
        let pw_hash = Argon2::default()
            .hash_password(plaintext_passwd.to_string().as_bytes(), &salt)?
            .to_string();
        self.passwd = pw_hash;
        Ok(())
    }

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
    /// 
    /// # Returns
    /// Success?
    pub async fn load(name: &str, plaintext_passwd: &str) -> Result<Player, LoadError> {
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
    async fn save(&mut self) -> Result<(), SaveError> {
        let filename = format!("{}/{}.save", *SAVE_PATH, self.name.slugify());
        let path = PathBuf::from_str(&filename).unwrap();
        let file = std::fs::File::create(path)?;
        let _ = serde_json::to_writer(file, &self)?;
        log::info!("Saved '{}'.", filename);
        Ok(())
    }
}

#[cfg(test)]
mod savefile_tests {
    use log::debug;

    use super::*;

    #[tokio::test]
    async fn create_new_savefile() {
        let mut s = Player::new("TestSaveThing");
        let r = s.set_passwd("new password, a very intricate thing");
        assert!(r.await.is_ok());
    }

    #[tokio::test]
    async fn save_savefile() {
        let _ = env_logger::try_init();
        let mut savefile = (*DUMMY_SAVE.as_ref()).clone();
        let _ = savefile.set_passwd("test word").await;
        let save_content = savefile.save().await;
        debug!("{:?}", save_content);
        assert!(save_content.is_ok());
    }

    #[tokio::test]
    async fn load_savefile() {
        let _ = env_logger::try_init();
        let savefile = Player::load("dummy", "test word").await;
        assert!(savefile.is_ok());
        debug!("{:?}", savefile);
    }

    #[tokio::test]
    async fn load_savefile_wrong_pwd() {
        let _ = env_logger::try_init();
        let savefile = Player::load("dummy", "wrong pwd").await;
        assert!(savefile.is_err());
    }
}
