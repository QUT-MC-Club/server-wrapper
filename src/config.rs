use std::collections::HashMap;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub use destinations::*;

mod destinations;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub run: Vec<String>,
    #[serde(default = "Default::default")]
    pub status: Status,
    #[serde(default = "Default::default")]
    pub tokens: Tokens,
    pub triggers: HashMap<String, Trigger>,
    #[serde(default = "default_min_restart_interval")]
    pub min_restart_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Status {
    pub webhook: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Tokens {
    pub github: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Trigger {
    #[serde(rename = "startup")]
    Startup,
    #[serde(rename = "webhook")]
    Webhook { port: u16 },
}

impl Default for Config {
    fn default() -> Self {
        Config {
            run: vec!["java -jar fabric-server-launch.jar".to_owned()],
            tokens: Tokens::default(),
            status: Status::default(),
            triggers: {
                let mut triggers = HashMap::new();
                triggers.insert("startup".to_owned(), Trigger::Startup);
                triggers
            },
            min_restart_interval_seconds: default_min_restart_interval(),
        }
    }
}

fn default_min_restart_interval() -> u64 {
    240
}

pub async fn load<P, T>(path: P) -> T
    where P: AsRef<Path>,
          T: Serialize + DeserializeOwned + Default
{
    let path = path.as_ref();
    if path.exists() {
        read_config(path).await.expect("failed to read config")
    } else {
        let config = T::default();
        write_config(path, &config).await.expect("failed to write default config");
        config
    }
}

async fn write_config<T: Serialize>(path: &Path, config: &T) -> io::Result<()> {
    let mut file = File::create(path).await?;

    let bytes = toml::to_vec(config).expect("malformed config");
    file.write_all(&bytes).await?;

    Ok(())
}

async fn read_config<T: DeserializeOwned>(path: &Path) -> io::Result<T> {
    let mut file = File::open(path).await?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).await?;

    Ok(toml::from_slice::<T>(&bytes).expect("malformed config"))
}
