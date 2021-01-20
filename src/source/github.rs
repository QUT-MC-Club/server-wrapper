use serde::Deserialize;

use crate::{cache, config, Error, Result, source};

pub async fn load<'a>(cache: cache::Entry<'a>, owner: &str, repository: &str, transform: &config::Transform) -> Result<cache::Reference> {
    let latest_artifact = get_latest_artifact(owner, repository).await?;

    if let Some((id, url, name)) = latest_artifact {
        use cache::UpdateResult::*;
        match cache.try_update(cache::Token::ArtifactId(id)) {
            Mismatch(updater) => {
                let name = format!("{}.zip", name);

                let url = reqwest::Url::parse(&url).unwrap();
                let response = octocrab::instance()._get(url, None::<&()>).await?;

                let bytes = response.bytes().await?;
                let file = source::File { name, bytes };

                if let Some(file) = transform.apply(file).await? {
                    Ok(updater.update(file).await?)
                } else {
                    Err(Error::MissingArtifact)
                }
            }
            Match(reference) => Ok(reference)
        }
    } else {
        cache.get_existing().ok_or(Error::MissingArtifact)
    }
}

async fn get_latest_artifact(owner: &str, repository: &str) -> Result<Option<(usize, String, String)>> {
    // TODO: we're not handling pagination, which means we rely on results being ordered by newest!

    let artifacts = get_artifacts(&owner, &repository).await?;
    let latest_artifact = artifacts.artifacts.into_iter()
        .filter(|artifact| !artifact.expired && artifact.archive_download_url.is_some())
        .max_by_key(|artifact| artifact.updated_at)
        .and_then(|artifact| {
            let id = artifact.id;
            let name = artifact.name;
            artifact.archive_download_url.map(|url| (id, url, name))
        });

    Ok(latest_artifact)
}

async fn get_artifacts(owner: &str, repository: &str) -> Result<ArtifactsResponse> {
    let octocrab = octocrab::instance();

    let route = format!("repos/{}/{}/actions/artifacts", owner, repository);
    Ok(octocrab.get(route, None::<&()>).await?)
}

#[derive(Deserialize, Debug)]
struct ArtifactsResponse {
    total_count: usize,
    artifacts: Vec<Artifact>,
}

#[derive(Deserialize, Debug)]
struct Artifact {
    id: usize,
    node_id: String,
    name: String,
    size_in_bytes: usize,
    url: String,
    archive_download_url: Option<String>,
    expired: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}
