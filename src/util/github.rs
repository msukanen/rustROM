use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GithubContent {
    pub name: String,
    #[serde(rename = "type")]
    pub content_type: String,
    pub download_url: Option<String>,
    pub url: String,
}
