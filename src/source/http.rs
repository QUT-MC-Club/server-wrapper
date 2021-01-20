use crate::{cache, config, Error, Result, source};

pub async fn load<'a>(cache: cache::Entry<'a>, url: &str, transform: &config::Transform) -> Result<cache::Reference> {
    let response = reqwest::get(url).await?;

    let etag = response.headers().get(reqwest::header::ETAG)
        .and_then(|etag| etag.to_str().ok());

    let cache_token = match etag {
        Some(etag) => cache::Token::Etag(etag[1..etag.len() - 1].to_owned()),
        None => cache::Token::Unknown,
    };

    use cache::UpdateResult::*;
    match cache.try_update(cache_token) {
        Mismatch(updater) => {
            println!("downloading {}...", url);

            let name = file_name(url).to_owned();
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
}

fn file_name(url: &str) -> &str {
    match url.rsplit_once("/") {
        Some((_, name)) => name,
        None => url,
    }
}
