use std::io;
use std::io::Read;

use bytes::Bytes;
use zip::ZipArchive;
use crate::source;

#[derive(Debug, Clone)]
pub enum Transform {
    Direct,
    Unzip { unzip: Vec<Pattern> },
}

impl Transform {
    pub async fn apply(&self, file: source::File) -> io::Result<Option<source::File>> {
        match self {
            Transform::Direct => Ok(Some(file)),
            Transform::Unzip { unzip } => apply_unzip(file, &unzip).await,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub glob: glob::Pattern,
    pub exclude: bool,
}

// TODO: potentially support loading multiple files + directories
async fn apply_unzip(
    file: source::File,
    patterns: &[Pattern],
) -> io::Result<Option<source::File>> {
    let patterns: Vec<Pattern> = patterns.iter().cloned().collect();

    tokio::task::spawn_blocking(move || {
        let cursor = io::Cursor::new(file.bytes.as_ref());
        let mut zip = ZipArchive::new(cursor)?;

        let jar_names: Vec<String> = zip
            .file_names()
            .filter(|path| matches_all(path, &patterns))
            .map(|path| path.to_owned())
            .collect();

        for name in jar_names {
            let mut file = zip.by_name(&name)?;
            if file.is_file() {
                let mut bytes = Vec::with_capacity(file.size() as usize);
                file.read_to_end(&mut bytes)?;

                let bytes = Bytes::from(bytes);
                return Ok(Some(source::File { name, bytes }));
            }
        }

        Ok(None)
    })
        .await
        .unwrap()
}

fn matches_all(path: &str, patterns: &[Pattern]) -> bool {
    let mut include = patterns.iter().filter(|pattern| !pattern.exclude);
    let mut exclude = patterns.iter().filter(|pattern| pattern.exclude);

    include.all(|pattern| pattern.glob.matches(path))
        && !exclude.any(|pattern| pattern.glob.matches(path))
}
