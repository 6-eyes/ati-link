use std::io::{Read, Write};
use super::error::Error;

use flate2::{read::{GzDecoder, ZlibDecoder}, write::{GzEncoder, ZlibEncoder}, Compression as Comp};

pub trait Compression {
    fn compress(&self, bytes: &[u8]) -> Result<Vec<u8>, Error>;
    fn decompress(&self, bytes: &[u8]) -> Result<Vec<u8>, Error>;
    fn get_type(&self) -> super::Compression;
}

pub struct Zlib;

impl Compression for Zlib {
    fn compress(&self, bytes: &[u8]) -> Result<Vec<u8>, Error> {
        let mut e = ZlibEncoder::new(Vec::new(), Comp::default());
        e.write_all(bytes)?;
        Ok(e.finish()?)
    }

    fn decompress(&self, bytes: &[u8]) -> Result<Vec<u8>, Error> {
        let d = ZlibDecoder::new(bytes);
        Ok(d.bytes().collect::<Result<Vec<u8>, std::io::Error>>()?)
    }

    fn get_type(&self) -> super::Compression {
        super::Compression::Zlib
    }
}

pub struct GZip;

impl Compression for GZip {
    fn compress(&self, bytes: &[u8]) -> Result<Vec<u8>, Error> {
        let mut e = GzEncoder::new(Vec::new(), Comp::default());
        e.write_all(bytes)?;
        Ok(e.finish()?)
    }

    fn decompress(&self, bytes: &[u8]) -> Result<Vec<u8>, Error> {
        let d = GzDecoder::new(bytes);
        Ok(d.bytes().collect::<Result<Vec<u8>, std::io::Error>>()?)
    }

    fn get_type(&self) -> super::Compression {
        super::Compression::GZip
    }
}

#[test]
fn zlib_test() {
    let marker = super::generate_eof_marker();

    let encrypted_result = Zlib.compress(&marker);
    assert!(encrypted_result.is_ok());

    let decrypted_result = Zlib.decompress(&encrypted_result.unwrap());
    assert!(decrypted_result.is_ok());

    assert_eq!(decrypted_result.unwrap(), marker);
}

#[test]
fn gzip_test() {
    let marker = super::generate_eof_marker();

    let encrypted_result = GZip.compress(&marker);
    assert!(encrypted_result.is_ok());

    let decrypted_result = GZip.decompress(&encrypted_result.unwrap());
    assert!(decrypted_result.is_ok());

    assert_eq!(decrypted_result.unwrap(), marker);
}
