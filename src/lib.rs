use std::fs;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ffi::OsString;

use walkdir;
use serde;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Entry {
    /// The path of this Entry relative to the indexed path
    relative_path: PathBuf,
    /// The chunked path, safe for comparison against another index path
    chunk: PathBuf,
    /// The file's size- not as affective as checksumming but doesn't require reading the whole
    /// file.
    size: u64,
}

#[derive(Debug)]
pub enum ChunkError {
    NoParent,
    NoFileName,
    DuplicateChunk(PathBuf),
}

impl fmt::Display for ChunkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // TODO(richo) None of this is super useful without context
            ChunkError::NoParent => write!(f, "Parent directory does not exist"),
            ChunkError::NoFileName => write!(f, "File name does not exist"),
            ChunkError::DuplicateChunk(path) => write!(f, "Duplicate chunk: {:?}", path),
        }
    }
}

impl std::error::Error for ChunkError {}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Db {
    entries: HashMap<PathBuf, Entry>,
}

impl Db {
    pub fn insert(&mut self, entry: Entry) -> Result<(), ChunkError> {
        // TODO(richo) make it give back on failure?
        let error_path = entry.relative_path.clone();
        if let Some(_) = self.entries.insert(entry.chunk.as_os_str().into(), entry) {
            return Err(ChunkError::DuplicateChunk(error_path));
        }

        Ok(())
    }

    pub fn write_to_file<F: std::io::Write>(&mut self, fh: F) -> Result<(), serde_json::Error> {
        serde_json::to_writer(fh, self)
    }

    pub fn read_from_file<F: std::io::Read>(fh: F) -> Result<Self, serde_json::Error> {
        serde_json::from_reader(fh)
    }
}

fn create_chunk(path: &Path) -> Result<PathBuf, ChunkError> {
    let mut out = PathBuf::new();
    // I think? Write some tests?
    out.push(path.parent().ok_or(ChunkError::NoParent)?.file_name().ok_or(ChunkError::NoParent)?);
    out.push(path.file_name().ok_or(ChunkError::NoFileName)?);
    Ok(out)
}

#[derive(Default)]
pub struct WalkerConfig {
    pub exclude: Vec<String>,
}

pub fn walk<T: AsRef<Path>>(path: T, cfg: WalkerConfig) -> impl Iterator<Item=Entry> {
    let WalkerConfig {
        exclude,
    } = cfg;

    let excludes = HashSet::<OsString, std::hash::RandomState>::from_iter(exclude.iter().map(|i| i.into()));

    walkdir::WalkDir::new(path)
        .into_iter()
        // TODO(richo) ??
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.file_type().is_file())
        .filter(move |e| !excludes.contains(e.file_name()))
        .map(|e| e.path().try_into())
        // TODO(richo) ??
        .filter_map(|e| e.ok())
}


impl TryInto<Entry> for &Path {
    type Error = std::io::Error;

    fn try_into(self) -> Result<Entry, Self::Error> {
        let metadata = fs::metadata(self)?;
        Ok(Entry {
            relative_path: self.to_path_buf(),
            chunk: create_chunk(self).unwrap(),
            size: metadata.len(),
        })
    }
}

impl TryInto<Entry> for PathBuf {
    type Error = std::io::Error;

    fn try_into(self) -> Result<Entry, Self::Error> {
        let metadata = fs::metadata(&self)?;
        Ok(Entry {
            relative_path: self.to_path_buf(),
            chunk: create_chunk(&self).unwrap(),
            size: metadata.len(),
        })
    }
}
