use std::io::{Read, Write};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression as Comp};

pub trait Compression {
    fn compress(bytes: &[u8]) -> Result<Vec<u8>, super::Error>;
    fn decompress(bytes: &[u8]) -> Result<Vec<u8>, super::Error>;
}

pub struct Zlib;

impl Compression for Zlib {
    fn compress(bytes: &[u8]) -> Result<Vec<u8>, super::Error> {
        let mut e = ZlibEncoder::new(Vec::new(), Comp::default());
        e.write_all(bytes)?;
        Ok(e.finish()?)
    }

    fn decompress(bytes: &[u8]) -> Result<Vec<u8>, super::Error> {
        let d = ZlibDecoder::new(bytes);
        Ok(d.bytes().collect::<Result<Vec<u8>, std::io::Error>>()?)
    }
}

#[test]
fn zlib_test() {
    let marker = super::generate_eof_marker();

    let encrypted_result = Zlib::compress(&marker);
    assert!(encrypted_result.is_ok());

    let decrypted_result = Zlib::decompress(&encrypted_result.unwrap());
    assert!(decrypted_result.is_ok());

    assert_eq!(decrypted_result.unwrap(), marker);
}
