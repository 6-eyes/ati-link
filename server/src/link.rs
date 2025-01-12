use std::{io::{Read, Write}, net::TcpStream};
use commons::{checksum::{Checksum, Sha256}, compression::{Compression, Zlib}, Error, CHUNK, Metadata};

pub struct Link {
    stream: TcpStream,
}

impl Link {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream
        }
    }

    /// method to send the file over the [`TcpStream`]
    pub fn send_file(&mut self, location: std::path::PathBuf, destination: std::path::PathBuf) -> Result<(), Error> {

        let metadata = self.upload_metadata(location.file_name().unwrap().to_str().unwrap(), destination)?;

        let file = std::fs::File::open(&location)?;
        let mut reader = std::io::BufReader::new(file);
        let mut buffer = [0; CHUNK];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                println!("reached end of file");
                self.upstream(&metadata.eof_marker)?;
                break;
            }

            self.upstream(&buffer)?;
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        Ok(())
    }

    /// method to upload the chunk
    /// - Calcualtes the checksum
    /// - Sends the length of the checksum
    /// - Sends the checksum
    /// - Compresses the chunk
    /// - Sends the length of the compressed chunk
    /// - Sends the compressed chunk
    fn upstream(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let checksum = Sha256.generate(&buffer);
        let checksum_bytes = checksum.as_bytes();
        self.write_len(checksum_bytes.len())?;
        self.stream.write_all(checksum_bytes)?;

        let buffer = Zlib::compress(buffer)?;
        self.write_len(buffer.len())?;
        self.stream.write_all(&buffer)?;

        Ok(())
    }

    /// method to write length to the stream
    fn write_len(&mut self, len: usize) -> Result<(), Error> {
        let len = len as u32;
        self.stream.write_all(len.to_be_bytes().as_ref())?;
        println!("written len: {}", len);
        Ok(())
    }


    /// method to upload metadata to the stream
    fn upload_metadata(&mut self, name: &str, destination: std::path::PathBuf) -> Result<Metadata, Error> {
        let metadata = Metadata::new(destination, name);
        let serialized = metadata.serialize();

        self.upstream(&serialized)?;
        Ok(metadata)
    }
}
