use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::source;

#[derive(Serialize, Deserialize, Default)]
struct Index {
    entries: Vec<IndexEntry>
}

async fn read_cache_index<P: AsRef<Path>>(path: P) -> io::Result<Index> {
    let path = path.as_ref();
    if path.exists() {
        let bytes = fs::read(path).await?;
        Ok(serde_json::from_slice(&bytes).expect("malformed cache index"))
    } else {
        Ok(Index::default())
    }
}

async fn write_cache_index<P: AsRef<Path>>(path: P, index: &Index) -> io::Result<()> {
    let path = path.as_ref();
    let bytes = serde_json::to_vec(index).expect("malformed cache index");
    fs::write(path, bytes).await
}

pub struct Loader {
    root: PathBuf,
    entries: HashMap<String, IndexEntry>,
}

impl Loader {
    pub async fn open<P: Into<PathBuf>>(path: P) -> io::Result<Loader> {
        let root = path.into();
        if !root.exists() {
            tokio::fs::create_dir_all(&root).await?;
        }

        let index = read_cache_index(&root.join("index.json")).await?;
        let entries = index.entries.into_iter()
            .map(|entry| (entry.key.clone(), entry))
            .collect();

        Ok(Loader { root, entries })
    }

    pub fn entry<K: Into<String>>(&mut self, key: K) -> Entry {
        let key = key.into();
        let current_token = self.entries.get(&key)
            .map(|entry| entry.token.clone())
            .unwrap_or(Token::Unknown);

        Entry { loader: self, key, current_token }
    }

    pub async fn close(self) -> io::Result<()> {
        let entries = self.entries.into_values().collect();
        write_cache_index(&self.root.join("index.json"), &Index { entries }).await?;
        Ok(())
    }

    async fn update_entry(&mut self, key: String, token: Token, name: String, bytes: &[u8]) -> io::Result<Reference> {
        let path = self.path_for(&key);

        fs::write(&path, bytes).await?;

        use std::collections::hash_map::Entry::*;

        match self.entries.entry(key.clone()) {
            Occupied(mut occupied) => {
                let occupied = occupied.get_mut();
                occupied.token = token;
                occupied.file_name = name.clone();
            }
            Vacant(vacant) => {
                vacant.insert(IndexEntry {
                    key,
                    token,
                    file_name: name.clone(),
                });
            }
        }

        Ok(Reference { path, name })
    }

    fn get_reference(&mut self, key: String) -> Option<Reference> {
        self.entries.get(&key).map(|entry| {
            let path = self.path_for(&key);
            let name = entry.file_name.clone();
            Reference { path, name }
        })
    }

    #[inline]
    fn path_for(&self, key: &str) -> PathBuf {
        self.root.join(key)
    }
}

#[derive(Serialize, Deserialize)]
struct IndexEntry {
    key: String,
    token: Token,
    file_name: String,
}

pub struct Entry<'a> {
    loader: &'a mut Loader,
    key: String,
    current_token: Token,
}

impl<'a> Entry<'a> {
    pub fn try_update(self, token: Token) -> UpdateResult<'a> {
        if self.current_token != token {
            println!("[{}] cache mismatched! new: {:?}, old: {:?}", self.key, token, self.current_token);
            UpdateResult::Mismatch(EntryUpdater {
                entry: self,
                token,
            })
        } else {
            println!("[{}] cache matched! {:?}", self.key, token);
            let reference = self.loader.get_reference(self.key.clone()).unwrap();
            UpdateResult::Match(reference)
        }
    }

    async fn update(&mut self, token: Token, name: String, bytes: &[u8]) -> io::Result<Reference> {
        self.loader.update_entry(self.key.clone(), token, name, bytes).await
    }
}

pub struct Reference {
    path: PathBuf,
    name: String,
}

impl Reference {
    pub async fn copy_to<P: AsRef<Path>>(&self, root: P) -> io::Result<()> {
        let target_path = root.as_ref().join(&self.name);
        fs::copy(&self.path, target_path).await?;
        Ok(())
    }
}

pub struct EntryUpdater<'a> {
    entry: Entry<'a>,
    token: Token,
}

impl<'a> EntryUpdater<'a> {
    pub async fn update(mut self, file: source::File) -> io::Result<Reference> {
        self.entry.update(self.token, file.name, file.bytes.as_ref()).await
    }
}

pub enum UpdateResult<'a> {
    Mismatch(EntryUpdater<'a>),
    Match(Reference),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Token {
    #[serde(rename = "etag")]
    Etag(String),
    #[serde(rename = "artifact")]
    ArtifactId(usize),
    #[serde(rename = "unknown")]
    Unknown,
}

impl PartialEq for Token {
    fn eq(&self, right: &Token) -> bool {
        use Token::*;
        match (self, right) {
            (Etag(left), Etag(right)) => left == right,
            (ArtifactId(left), ArtifactId(right)) => left == right,
            (_, _) => false
        }
    }
}

impl Eq for Token {}
