pub mod checksum;
pub mod compression;
pub mod error;
pub mod connection;

use std::sync::LazyLock;
use rand::{rngs::OsRng, RngCore};

pub static EOF_MARKER: LazyLock<[u8; CHUNK]> = LazyLock::new(generate_eof_marker);
pub const CHUNK: usize = 10;

fn generate_eof_marker() -> [u8; CHUNK] {
    println!("GENERATING MARKER");
    let mut rng = OsRng;
    let mut marker = [0u8; CHUNK];
    rng.fill_bytes(&mut marker);
    marker
}

/// Metadata to be sent at the start of each file
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FileMetadata {
    pub rel_path: std::path::PathBuf,
}

impl FileMetadata {
    pub fn new(path: &std::path::Path) -> Self {
        Self {
            rel_path: path.to_path_buf(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}

/// Metadata sent before the initiation of file transfer
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UploadMetadata {
    pub count: u32,
    pub destination: std::path::PathBuf,
    pub eof_marker: Vec<u8>,
    pub compression: Option<Compression>,
    pub checksum: Option<Checksum>,
}

impl UploadMetadata {
    pub fn new(count: u32, destination: &std::path::Path) -> Self {
        Self {
            count,
            destination: destination.to_path_buf(),
            eof_marker: EOF_MARKER.to_vec(),
            compression: None,
            checksum: None,
        }
    }

    pub fn with_compression(self, compression: Option<Compression>) -> Self {
        Self {
            compression,
            ..self
        }
    }

    pub fn with_checksum(self, checksum: Option<Checksum>) -> Self {
        Self {
            checksum,
            ..self
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}

/// Metadata sent by client for receiving data
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DownloadMetadata {
    pub destination: std::path::PathBuf,
    pub compression: Option<Compression>,
    pub checksum: Option<Checksum>,
}

impl DownloadMetadata {
    pub fn new(destination: &std::path::Path) -> Self {
        Self {
            destination: destination.to_path_buf(),
            compression: None,
            checksum: None,
        }
    }

    pub fn with_compression(self, compression: Option<Compression>) -> Self {
        Self {
            compression,
            ..self
        }
    }

    pub fn with_checksum(self, checksum: Option<Checksum>) -> Self {
        Self {
            checksum,
            ..self
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}

/// Result send by server before initializing file transfer
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Result {
    Marker {
        count: u32,
        marker: Vec<u8>,
    },
    Err(String),
}

impl Result {
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Compression {
    Zlib,
    GZip,
}

impl Compression {
    pub fn get_algo(&self) -> Box<dyn compression::Compression> {
        match self {
            Self::GZip => Box::new(compression::GZip),
            Self::Zlib => Box::new(compression::Zlib),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Checksum {
    Sha256,
    Md5,
}

impl Checksum {
    pub fn get_algo(&self) -> Box<dyn checksum::Checksum> {
        match self {
            Self::Sha256 => Box::new(checksum::Sha256),
            Self::Md5 => Box::new(checksum::Md5),
        }
    }
}

/// The role assigned to the server
#[derive(Debug, serde::Serialize, serde::Deserialize, Copy, Clone)]
pub enum Role {
    Source,
    Sink,
}

impl Role {
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}

/// method to recursively get all the files in the directory tree
pub fn get_recursive_paths(path: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let metadata = std::fs::symlink_metadata(path).unwrap();
    if metadata.is_file() {
        files.push(std::path::PathBuf::from(path));
    }
    else if metadata.is_dir() {
        for file in std::fs::read_dir(path).unwrap() {
            let path = file.unwrap().path();
            files.extend(get_recursive_paths(&path));
        }
    }

    files
}
