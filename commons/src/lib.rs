pub mod checksum;
pub mod compression;

use std::{io::Read, net::AddrParseError, sync::LazyLock};

use rand::{rngs::OsRng, RngCore};

static EOF_MARKER: LazyLock<[u8; CHUNK]> = LazyLock::new(generate_eof_marker);
pub const CHUNK: usize = 16 * 1024;

fn generate_eof_marker() -> [u8; CHUNK] {
    let mut rng = OsRng;
    let mut marker = [0u8; CHUNK];
    rng.fill_bytes(&mut marker);
    marker
}

/// Metadata sent before the initiation of file transfer
#[derive(Default, Debug)]
pub struct Metadata {
    pub path: std::path::PathBuf,
    pub eof_marker: Vec<u8>,
}

const DELIMITER: char = '^';

impl Metadata {
    pub fn new(path: std::path::PathBuf, name: &str) -> Self {
        Self {
            path: path.join(name),
            eof_marker: EOF_MARKER.to_vec(),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut result = String::new();

        result.push_str("path?");
        result.push_str(&self.path.to_string_lossy());
        result.push(DELIMITER);

        result.push_str("eof_marker?");
        let str = self.eof_marker.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(",");
        result.push_str(&str);

        result.into_bytes()
    }

    pub fn deserialize(mut bytes: &[u8]) -> Result<Self, Error> {
        let mut s = String::new();
        bytes.read_to_string(&mut s)?;

        let split = s.split(DELIMITER);

        let mut metadata = Metadata::default();
        for kv_pair in split {
            let (k, v) = kv_pair.split_once('?').unwrap();
            match k {
                "path" => metadata.path = std::path::PathBuf::from(v),
                "eof_marker" => metadata.eof_marker = v.split(',').map(|s| s.parse::<u8>().unwrap()).collect(),
                _ => return Err(Error::Deserialize),
            }
        }

        Ok(metadata)
    }
}


#[derive(Debug)]
pub enum Error {
    InvalidAddress,
    InvalidArgument(String),
    IO(std::io::Error),
    Deserialize,
    ConnectionFailed,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAddress => write!(f, "invalid address input"),
            Self::InvalidArgument(s) => write!(f, "{}", s),
            Self::IO(e) => write!(f, "io error {e}"),
            Self::Deserialize => write!(f, "unable to parse string"),
            Self::ConnectionFailed => write!(f, "unable to connect"),
        }
    }
}

impl From<AddrParseError> for Error {
    fn from(value: AddrParseError) -> Self {
        eprintln!("{value}");
        Self::InvalidAddress
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(_value: std::num::ParseIntError) -> Self {
        Self::Deserialize
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{Metadata, EOF_MARKER};

    #[test]
    fn serialization_and_deserialization_of_metadata() {
        let path = "server/src/lib.rs";
        let name = "name";
        let metadata = Metadata::new(std::path::PathBuf::from(path), name);

        let serialized = metadata.serialize();
        let deserialized = Metadata::deserialize(&serialized);

        assert!(deserialized.is_ok());

        let metadata = deserialized.unwrap();
        assert!(metadata.path.to_str().is_some());

        assert!(metadata.path.to_str().unwrap() == format!("{path}/{name}") && metadata.eof_marker == EOF_MARKER.clone());
    }
}
