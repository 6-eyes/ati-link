use std::{io::{Read, Write}, net::TcpStream};
use commons::{checksum::{Checksum, Sha256}, compression::{Compression, Zlib}, Error, Metadata};

pub struct Link {
    stream: TcpStream,
}

impl Link {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
        }
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        let metadata = self.fetch_metadata()?;

        if let Some(parent) = metadata.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = std::fs::File::create(&metadata.path)?;

        loop {
            match self.downstream() {
                Ok(buffer) if buffer == metadata.eof_marker => {
                    tracing::info!("reached end of file");
                    break;
                }
                Ok(buffer) => file.write_all(&buffer)?,
                Err(e) => {
                    tracing::error!("error reading chunk {e}. Deleting file at {}", metadata.path.to_str().unwrap());
                    std::fs::remove_file(&metadata.path)?;
                    break;
                },
            }
        }

        Ok(())
    }

    fn fetch_metadata(&mut self) -> Result<Metadata, Error> {
        let metadata_bytes = self.downstream()?;
        Ok(Metadata::deserialize(&metadata_bytes)?)
    }

    /// method to read an incoming chunk
    /// - Reads the length of the checksum
    /// - Reads the checksum
    /// - Reads the length of the chunk
    /// - Reads the chunk
    /// - Decrypts the buffer
    /// - Validate checksum
    fn downstream(&mut self) -> Result<Vec<u8>, Error> {
        let checksum = {
            let checksum_len = self.read_len()?;
            let mut checksum_bytes = vec![0; checksum_len as usize];
            let bytes_read = self.stream.read(&mut checksum_bytes)?;

            if bytes_read == 0 {
                todo!("return checksum not found error");
            }
            match String::from_utf8(checksum_bytes) {
                Ok(s) => s,
                Err(e) => todo!("return checksum conversion error"),
            }
        };

        let chunk = {
            let chunk_len = self.read_len()?;
            let mut buffer = vec![0; chunk_len as usize];
            let bytes_read = self.stream.read(&mut buffer)?;
            tracing::info!("read bytes: {bytes_read}");

            Zlib::decompress(&buffer)?
        };

        match Sha256.valdate(&chunk, &checksum) {
            true => {
                tracing::info!("checksum passed");
                Ok(chunk)
            },
            false => todo!("return integrity error"),
        }
    }

    /// Method to read the first 4 bytes of a stream.
    /// Used to determine the length of the incoming message
    fn read_len(&mut self) -> Result<u32, Error> {
        let mut len_buf = [0; 4];
        self.stream.read_exact(&mut len_buf)?;

        let payload_len = u32::from_be_bytes(len_buf);
        tracing::info!("read len: {payload_len}");
        Ok(payload_len)
    }
}
