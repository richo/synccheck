use std::fs;
use std::path::{Path, PathBuf};

use walkdir;

#[derive(Debug)]
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
}

fn create_chunk(path: &Path) -> Result<PathBuf, ChunkError> {
    let mut out = PathBuf::new();
    // I think? Write some tests?
    out.push(path.parent().ok_or(ChunkError::NoParent)?.file_name().ok_or(ChunkError::NoParent)?);
    out.push(path.file_name().ok_or(ChunkError::NoFileName)?);
    Ok(out)
}

pub fn walk<T: AsRef<Path>>(path: T) -> impl Iterator<Item=Entry> {
    walkdir::WalkDir::new(path)
        .into_iter()
        // TODO(richo) ??
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
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
