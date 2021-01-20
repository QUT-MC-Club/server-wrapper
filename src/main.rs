#![feature(str_split_once)]
#![feature(map_into_keys_values)]

use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use octocrab::OctocrabBuilder;
use tokio::fs;

pub use config::Config;
use executor::Executor;
use status::StatusWriter;
use futures::FutureExt;
use std::collections::HashMap;

mod cache;
mod config;
mod executor;
mod status;
mod source;

const CACHE_ROOT: &str = "wrapper_cache";

const MIN_RESTART_INTERVAL: Duration = Duration::from_secs(4 * 60);

// TODO: implement triggers

#[tokio::main]
pub async fn main() {
    loop {
        let config: Config = config::load("config.toml").await;
        let destinations: config::Destinations = config::load("destinations.toml").await;

        let status = match config.status.webhook {
            Some(webhook) => StatusWriter::from(status::webhook::Client::open(webhook)),
            None => StatusWriter::none(),
        };

        let mut octocrab = OctocrabBuilder::new();
        if let Some(github) = config.tokens.github {
            octocrab = octocrab.personal_token(github);
        }
        octocrab::initialise(octocrab).expect("failed to initialize github api");

        let destinations: Vec<PreparedDestination> = prepare_destinations(destinations.destinations, &status).await;
        for destination in destinations {
            destination.apply().await.expect("failed to apply destination");
        }

        status.write("Starting up server...");

        let start = Instant::now();

        let mut executor = Executor::new(config.run);
        if let Err(err) = executor.run().await {
            eprintln!("server exited with error: {:?}", err);
        } else {
            println!("server closed");
        }

        let interval = Instant::now() - start;
        if interval < MIN_RESTART_INTERVAL {
            println!("server restarted very quickly! waiting a bit...");

            let delay = MIN_RESTART_INTERVAL - interval;
            status.write(format!("Server restarted too quickly! Waiting for {} seconds...", delay.as_secs()));

            tokio::time::sleep(delay.into()).await;
        } else {
            status.write("Server closed! Restarting...");
        }
    }
}

async fn prepare_destinations(destinations: HashMap<String, config::Destination>, status: &StatusWriter) -> Vec<PreparedDestination> {
    let mut futures = Vec::new();

    for (destination_name, destination) in destinations {
        let status = status.clone();
        let future = tokio::spawn(async move {
            prepare_destination(&destination_name, &destination, &status).await
                .expect(&format!("failed to prepare destination '{}'", destination_name))
        });
        futures.push(future.map(|result| result.unwrap()));
    }

    futures::future::join_all(futures).await
}

// TODO: load sources concurrently
async fn prepare_destination(destination_name: &str, destination: &config::Destination, status: &StatusWriter) -> Result<PreparedDestination> {
    let cache_root = Path::new(CACHE_ROOT).join(destination_name);

    let mut cache_files = Vec::with_capacity(destination.sources.len());

    let mut cache = cache::Loader::open(&cache_root).await?;
    for (_, source_set) in &destination.sources {
        for (key, source) in &source_set.sources {
            let cache_entry = cache.entry(key.clone());
            match source::load(cache_entry, source, &source_set.transform).await {
                Ok(reference) => cache_files.push(reference),
                Err(err) => {
                    eprintln!("failed to load {}: {:?}! excluding.", key, err);
                    status.write(format!("Failed to load {}... Excluding!", key));
                }
            }
        }
    }

    cache.close().await?;

    Ok(PreparedDestination {
        root: destination.path.clone(),
        cache_files,
    })
}

struct PreparedDestination {
    root: PathBuf,
    cache_files: Vec<cache::Reference>,
}

impl PreparedDestination {
    async fn apply(&self) -> Result<()> {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).await?;
        }

        fs::create_dir_all(&self.root).await?;

        for reference in &self.cache_files {
            reference.copy_to(&self.root).await?;
        }

        Ok(())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("zip error")]
    Zip(#[from] zip::result::ZipError),
    #[error("http error")]
    Reqwest(#[from] reqwest::Error),
    #[error("github error")]
    Octocrab(#[from] octocrab::Error),
    #[error("malformed github reference")]
    MalformedGitHubReference(String),
    #[error("missing artifact")]
    MissingArtifact,
}
