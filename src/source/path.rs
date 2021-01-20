use std::path::PathBuf;

use bytes::Bytes;
use tokio::fs;

use crate::{cache, config, Error, Result, source};

pub async fn load<'a>(cache: cache::Entry<'a>, path: &PathBuf, transform: &config::Transform) -> Result<cache::Reference> {
    use cache::UpdateResult::*;
    match cache.try_update(cache::Token::Unknown) {
        Mismatch(updater) => {
            let name = path.file_name().and_then(|name| name.to_str()).unwrap().to_owned();

            let bytes = fs::read(&path).await?;
            let bytes = Bytes::from(bytes);

            let file = source::File { name, bytes };

            if let Some(file) = transform.apply(file).await? {
                Ok(updater.update(file).await?)
            } else {
                Err(Error::MissingArtifact)
            }
        }
        Match(reference) => Ok(reference)
    }
}
