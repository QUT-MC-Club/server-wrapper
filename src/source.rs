use bytes::Bytes;

use crate::{Error, Result};
use crate::cache;
use crate::config::{self, Source};

mod github;
mod http;

pub async fn load<'a>(cache: cache::Entry<'a>, source: &config::Source, transform: &config::Transform) -> Result<cache::Reference> {
    match source {
        Source::GitHubArtifacts { github } => {
            match github.split("/").collect::<Vec<&str>>().as_slice() {
                [owner, repository] => github::load(cache, owner, repository, transform).await,
                _ => Err(Error::MalformedGitHubReference(github.clone())),
            }
        }
        Source::Url { url } => http::load(cache, url, transform).await
    }
}

pub struct File {
    pub name: String,
    pub bytes: Bytes,
}
