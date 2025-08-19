use std::{fs::OpenOptions, io::Write, sync::Arc};

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::{string::WordSet, util::GithubContent, DATA_PATH};

static GITHUB_BAD_NAMES_REPO_URL: &str = "https://api.github.com/repos/msukanen/english-words/contents";
pub(crate) static BAD_NAMES_FILEPATH: Lazy<String> = Lazy::new(|| format!("{}/badwords.txt", *DATA_PATH));

/// Filter 'bad names'.
/// 
/// Bad names can be anything really:
/// - something vulgar,
/// - a reserved word,
/// - name of a place or a person,
/// - name of famous character in some game or book,
/// - … etc.
/// 
/// # Arguments
/// - `name` to check.
/// 
/// # Returns
/// `true` if given `name` is considered to be a Bad Name™.
pub async fn filter_bad_name(badname_lock: Arc<RwLock<WordSet>>, name: &str) -> bool {
    if badname_lock.read().await.contains(&name.trim().to_lowercase()) { return true; }

    let checks: Vec<fn(&str) -> bool> = vec![
        is_reserved_name,
        is_reserved_word,
    ];

    checks.iter().any(|f| f(name))
}

fn is_reserved_name(name: &str) -> bool {
    matches!(name.to_lowercase().as_str(),
        "ani"|"anita"|"anikaiful"|"anikai"|"anitakai"|"asimov"|
        "hjyrok"|"hjyroku"|"hjyng"|
        "linus"|"torvalds"|
        "msukanen"|"markku"|"sukanen"|
        "kataract"|"thekataract"|
        "soto"
    )
}

fn is_reserved_word(name: &str) -> bool {
    matches!(name.to_lowercase().as_str(),
        "admin"|"alien"|
        "fn"|
        "linux"|
        "mysql"|
        "root"|
        "system"|"sql"|
        "unix"|"ufo"
    )
}

/// Recursively fetches bad name files from a GitHub repository.
async fn fetch_bad_names_from_github(repo_url: &str, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().user_agent("RustROM MUD").build()?;
    let response = client.get(repo_url).send().await?.json::<Vec<GithubContent>>().await?;

    let mut file = OpenOptions::new().append(true).create(true).open(filepath)?;

    for item in response {
        if item.content_type == "file" && item.name.as_str() == "words.txt" {
            if let Some(download_url) = item.download_url {
                log::info!("  → downloading {}…", item.name);
                let content = client.get(&download_url).send().await?.text().await?;
                file.write_all(content.as_bytes())?;
                file.write_all(b"\n")?; // Ensure a newline between files
            }
        } else if item.content_type == "dir" {
            log::debug!("Recursing '{:?}'", item);
            // Recursively fetch from the subdirectory
            Box::pin(fetch_bad_names_from_github(&item.url, filepath)).await?;
        }
    }

    Ok(())
}

pub(crate) async fn load_bad_names(path: &str) -> WordSet {
    match std::fs::exists(path) {
        Ok(true) => {
            let contents = std::fs::read_to_string(path).expect(&format!("Error loading '{}'", path));
            let mut words = WordSet::new();
            log::info!("Processing {} bad names…", contents.lines().count());
            for line in contents.lines() {
                let w = line.trim();
                if !w.is_empty() {
                    words.insert(w.to_lowercase());
                }
            }
            log::info!("Remaining {} bad names after normalization.", words.len());
            words
        }
        _ => {
            fetch_bad_names_from_github(GITHUB_BAD_NAMES_REPO_URL, path)
                .await
                .expect("Error fetching from github!");
            Box::pin(load_bad_names(path)).await
        }
    }
}

#[cfg(test)]
mod badname_tests {
    use crate::{util::badname::{load_bad_names, BAD_NAMES_FILEPATH}, DATA};

    #[tokio::test]
    async fn fetch_badnames() {
        let _ = env_logger::try_init();
        let _ = DATA.set("./data".into());
        let bw = load_bad_names(&BAD_NAMES_FILEPATH).await;
        assert!(bw.contains("abc"));
    }
}
