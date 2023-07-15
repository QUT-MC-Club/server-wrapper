use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;

use crate::transform;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Destinations {
    pub destinations: HashMap<String, Destination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Destination {
    pub path: PathBuf,
    pub triggers: Vec<String>,
    pub sources: HashMap<String, SourceSet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSet {
    #[serde(default = "Default::default")]
    pub transform: Option<Transform>,
    #[serde(flatten)]
    pub sources: HashMap<String, Source>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Transform {
    Unzip { unzip: Vec<transform::Pattern> },
}

impl Serialize for transform::Pattern {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.exclude {
            serializer.serialize_str(&format!("!{}", self.glob))
        } else {
            serializer.serialize_str(self.glob.as_str())
        }
    }
}

impl<'de> Deserialize<'de> for transform::Pattern {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut pattern: &str = Deserialize::deserialize(deserializer)?;
        let mut exclude = false;
        if pattern.starts_with("!") {
            pattern = &pattern[1..];
            exclude = true;
        }

        match glob::Pattern::new(pattern) {
            Ok(glob) => Ok(transform::Pattern { glob, exclude }),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Source {
    GitHubArtifacts {
        github: String,
        workflow: Option<String>,
        branch: Option<String>,
        artifact: Option<String>,
    },
    Modrinth {
        project_id: String,
        game_version: Option<String>,
    },
    Url {
        url: String,
    },
    Path {
        path: PathBuf,
    },
}

impl Default for Destinations {
    fn default() -> Self {
        let mut destinations = HashMap::new();
        destinations.insert("mods".to_owned(), Destination {
            path: PathBuf::from("mods"),
            triggers: vec!["startup".to_owned()],
            sources: {
                let mut sources = HashMap::new();
                sources.insert("fabric-api".to_owned(), Source::Url {
                    url: "https://github.com/FabricMC/fabric/releases/download/0.29.3%2B1.16/fabric-api-0.29.3+1.16.jar".to_owned()
                });

                let mut source_sets = HashMap::new();
                source_sets.insert("jars".to_owned(), SourceSet {
                    transform: None,
                    sources,
                });

                source_sets
            },
        });

        Destinations { destinations }
    }
}
