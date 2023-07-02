use crate::cache;
use crate::config;
use crate::source;
use crate::Error;
use crate::Result;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;

pub async fn load<'a>(
    client: &Client,
    cache: cache::Entry<'a>,
    project_id: &str,
    game_version: &Option<String>,
    transform: &config::Transform,
) -> Result<cache::Reference> {
    let latest_version = resolve_version(client, project_id, game_version).await?;
    if let Some((hash, url, name)) = latest_version {
        use cache::UpdateResult::*;
        match cache.try_update(cache::Token::Sha512(hash)) {
            Mismatch(updater) => {
                let name = format!("{}.zip", name);

                let response = client.get(&url).await?;
                let bytes = response.bytes().await?;
                let file = source::File { name, bytes };

                if let Some(file) = transform.apply(file).await? {
                    Ok(updater.update(file).await?)
                } else {
                    Err(Error::MissingArtifact)
                }
            }
            Match(reference) => Ok(reference),
        }
    } else {
        cache.get_existing().ok_or(Error::MissingArtifact)
    }
}

async fn resolve_version(
    client: &Client,
    project_id: &str,
    game_version: &Option<String>,
) -> Result<Option<(String, String, String)>> {
    let mut versions = client.get_versions(project_id, game_version).await?;
    versions.sort_by_key(|v| v.date_published);
    for version in versions {
        let file = version.files.iter().filter(|f| f.primary).next();
        if let Some(file) = file {
            if let Some(hash) = &file.hashes.sha512 {
                return Ok(Some((
                    hash.clone(),
                    file.url.clone(),
                    file.filename.clone(),
                )));
            } else {
                eprintln!("Warning: encountered old mod version without sha512 hash, skipping");
            }
        }
    }

    return Ok(None);
}

#[derive(Clone)]
pub struct Client {
    client: Arc<reqwest::Client>,
}

impl Client {
    const BASE_URL: &'static str = "https://api.modrinth.com";

    pub fn new(client: reqwest::Client) -> Client {
        Client {
            client: Arc::new(client),
        }
    }

    async fn get_versions(
        &self,
        project_id: &str,
        game_version: &Option<String>,
    ) -> Result<Vec<ProjectVersion>> {
        let url = if let Some(game_version) = game_version {
            format!(
                "{}/v2/project/{}/version?game_version={}",
                Client::BASE_URL,
                project_id,
                game_version
            )
        } else {
            format!("{}/v2/project/{}/version", Client::BASE_URL, project_id)
        };
        let response = self.get(&url).await?;
        Ok(response.json().await?)
    }

    #[inline]
    pub async fn get(&self, url: &str) -> Result<reqwest::Response> {
        Ok(self.client.get(url).send().await?)
    }
}

#[derive(Deserialize, Debug)]
pub struct ProjectVersion {
    date_published: DateTime<Utc>,
    files: Vec<ProjectFile>,
}

#[derive(Deserialize, Debug)]
pub struct ProjectFile {
    url: String,
    filename: String,
    primary: bool,
    hashes: FileHashes,
}

#[derive(Deserialize, Debug)]
pub struct FileHashes {
    sha512: Option<String>,
}
