use sha1::{Digest, Sha1};

#[derive(Debug)]
pub enum PasswordError {
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

/// Check pwd pwnage status via HIBP.
/// 
/// # Arguments
/// - `plaintext_password`— password (or alike) to check.
/// 
/// # Returns
/// Either `Ok(())` or e.g. `Err(`[PasswordError::HIBPPwned]`)`.
pub async fn is_passwd_pwned(plaintext_passwd: &str) -> Result<(), PasswordError> {
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
pub async fn validate_passwd(plaintext_passwd: &str) -> Result<(), PasswordError>
{
    if plaintext_passwd.len() < MIN_PASSWD_LENGTH { return Err(PasswordError::TooShort);}
    if !plaintext_passwd.chars().any(|c| c.is_ascii_lowercase()) {return Err(PasswordError::NoLowercase);}
    if !plaintext_passwd.chars().any(|c| c.is_ascii_uppercase()) {return Err(PasswordError::NoUppercase);}
    if !plaintext_passwd.chars().any(|c| c.is_ascii_digit()) {return Err(PasswordError::NoDigit);}
    if !plaintext_passwd.chars().any(|c| c.is_alphanumeric()) {return Err(PasswordError::NoSpecial);}
    is_passwd_pwned(plaintext_passwd).await
}
