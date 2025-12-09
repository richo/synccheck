use std::fs;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ffi::OsString;

use walkdir;
use serde;

const SIGNIFICANT_CHUNKS: usize = 2;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Entry {
    /// The path of this Entry relative to the indexed path
    pub relative_path: PathBuf,
    /// The chunked path, safe for comparison against another index path
    chunk: PathBuf,
    /// The file's size- not as affective as checksumming but doesn't require reading the whole
    /// file.
    size: u64,
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum DiffError {
    MismatchedChunks,
}

impl fmt::Display for DiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // TODO(richo) None of this is super useful without context
            DiffError::MismatchedChunks => write!(f, "Diff files have different chunk size"),
        }
    }
}


impl std::error::Error for ChunkError {}
impl std::error::Error for DiffError {}

#[derive(Debug, Default)]
pub struct DbDiffs {
    missing: Vec<Entry>,
    mismatched_size: Vec<Entry>,
}

impl DbDiffs {
    pub fn out_of_sync(&self) -> bool {
        return self.missing.len() > 0 ||
            self.mismatched_size.len() > 0;
    }

    pub fn missing(&self) -> &[Entry] {
        &self.missing[..]
    }

    pub fn mismatched_size(&self) -> &[Entry] {
        &self.mismatched_size[..]
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Db {
    significant_chunks: usize,
    entries: HashMap<PathBuf, Entry>,
}

impl Default for Db {
    fn default() -> Self {
        Db {
            significant_chunks: SIGNIFICANT_CHUNKS,
            entries: Default::default(),
        }
    }
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

    pub fn diffs_from(&self, other: &Self) -> Result<DbDiffs, DiffError> {
        // TODO(richo) I think in theory we could still try to compare these
        if self.significant_chunks != other.significant_chunks {
            Err(DiffError::MismatchedChunks)?;
        }

        let mut diffs = DbDiffs::default();
        for (_, entry) in other.entries.iter() {
            if let Some(v) = self.entries.get(&entry.chunk) {
                if v.size != entry.size {
                    diffs.mismatched_size.push(entry.to_owned());
                }
            } else {
                diffs.missing.push(entry.to_owned());
            }
        }

        Ok(diffs)
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
    let mut parts = Vec::with_capacity(SIGNIFICANT_CHUNKS);
    let mut cur = path;

    for _ in 0..SIGNIFICANT_CHUNKS {
        let parent = match cur.parent() {
            Some(par) => par,
            None => break,
        };
        parts.push(parent.file_name().ok_or(ChunkError::NoParent)?);
        cur = parent;
    }

    while let Some(part) = parts.pop() {
        out.push(part);
    }

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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_deep_chunk() {
        let pb = PathBuf::from("/root/thing/whatever/file.txt");
        assert_eq!(
            create_chunk(&pb),
            Ok(PathBuf::from("thing/whatever/file.txt"))
        );
    }
}
