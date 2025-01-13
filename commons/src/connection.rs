use std::{io::{Read, Write}, net::TcpStream, path};
use crate::{DownloadMetadata, Role, UploadMetadata};

use super::{CHUNK, EOF_MARKER, checksum, compression, error, FileMetadata};

pub struct Link {
    stream: TcpStream,
    compression: Option<Box<dyn compression::Compression>>,
    checksum: Option<Box<dyn checksum::Checksum>>,
}

impl Link {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            compression: None,
            checksum: None,
        }
    }

    pub fn with_compression(self, compression: Option<Box<dyn compression::Compression>>) -> Self {
        if compression.is_none() {
            tracing::info!("COMPRESSION is NONE");
        }
        Self {
            compression,
            ..self
        }
    }

    pub fn with_checksum(self, checksum: Option<Box<dyn checksum::Checksum>>) -> Self {
        if checksum.is_none() {
            tracing::info!("CHECKSUM is NONE");
        }
        Self {
            checksum,
            ..self
        }
    }
}

/// Methods aimed for reading from stram
impl Link {
    pub fn read_from_stream(&mut self, destination: &path::Path, marker: &[u8]) -> Result<(), error::Error> {
        tracing::info!("reading file metadata");
        let file_metadata = self.read_file_metadata()?;
        let path = destination.join(file_metadata.rel_path);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = std::fs::File::create(&path)?;

        loop {
            match self.downstream() {
                Ok(buffer) if buffer == marker => {
                    tracing::info!("reached end of file");
                    break;
                }
                Ok(buffer) => file.write_all(&buffer)?,
                Err(e) => {
                    tracing::error!("error reading chunk {e}. Deleting file at {}", path.to_str().unwrap());
                    std::fs::remove_file(&path)?;
                    tracing::info!("file {} removed successfully", path.to_str().unwrap());
                    break;
                },
            }
        }

        Ok(())
    }

    /// method to read an incoming chunk
    /// - Reads the length of the checksum
    /// - Reads the checksum
    /// - Reads the length of the chunk
    /// - Reads the chunk
    /// - Decrypts the buffer
    /// - Validate checksum
    fn downstream(&mut self) -> Result<Vec<u8>, error::Error> {
        let mut checksum = None;
        if self.checksum.is_some() {
            let checksum_len = self.read_len()?;
            tracing::debug!("read checksum len: {}", checksum_len);
            let mut checksum_bytes = vec![0; checksum_len as usize];
            let bytes_read = self.stream.read(&mut checksum_bytes)?;

            if bytes_read == 0 {
                let err = "checksum bytes not determined";
                tracing::error!("{}", err);
                return Err(error::Error::integrity_error(err));
            }

            checksum = String::from_utf8(checksum_bytes).ok();
        }

        let chunk = {
            let chunk_len = self.read_len()?;
            tracing::info!("reading bytes: {}", chunk_len);
            let mut buffer = vec![0; chunk_len as usize];
            self.stream.read_exact(&mut buffer)?;

            match &self.compression {
                Some(algo) => algo.decompress(&buffer)?,
                None => buffer,
            }
        };

        if let Some(algo) = &self.checksum {
            match checksum {
                None => {
                    let err = "unable to parse checksum bytes";
                    tracing::error!("{err}");
                    Err(error::Error::integrity_error(err))
                },
                Some(hash) => match algo.valdate(&chunk, &hash) {
                    true => {
                        tracing::info!("checksum passed");
                        Ok(chunk)
                    },
                    false => {
                        let err = "checksum verification failed";
                        tracing::error!("{err}");
                        Err(error::Error::integrity_error(err))
                    },
                },
            }
        }
        else {
            Ok(chunk)
        }
    }

    /// Method to read the first 4 bytes of a stream.
    /// Used to determine the length of the incoming message
    fn read_len(&mut self) -> Result<u32, error::Error> {
        let mut len_buf = [0; 4];
        self.stream.read_exact(&mut len_buf)?;

        let payload_len = u32::from_be_bytes(len_buf);
        Ok(payload_len)
    }

    /// reader for [`DownloadMetadata`]
    pub fn read_download_metadata(&mut self) -> Result<DownloadMetadata, error::Error> {
        let len = self.read_len()?;
        let mut buffer = vec![0; len as usize];

        self.stream.read_exact(&mut buffer)?;

        let download_metadata = DownloadMetadata::from_bytes(&buffer);

        tracing::debug!("received download metadata: {:?}", download_metadata);
        self.compression = download_metadata.compression.as_ref().map(|c| c.get_algo());
        self.checksum = download_metadata.checksum.as_ref().map(|c| c.get_algo());

        Ok(download_metadata)
    }

    /// method to read role from a stream
    pub fn read_role(&mut self) -> Result<Role, error::Error> {
        let len = self.read_len()?;
        let mut buffer = vec![0; len as usize];

        self.stream.read_exact(&mut buffer)?;
        Ok(Role::from_bytes(&buffer))
    }

    pub fn read_result(&mut self) -> Result<super::Result, error::Error> {
        let bytes = self.downstream()?;
        Ok(super::Result::from_bytes(&bytes))
    }

    fn read_file_metadata(&mut self) -> Result<FileMetadata, error::Error> {
        let bytes = self.downstream()?;
        Ok(super::FileMetadata::from_bytes(&bytes))
    }

    /// raw read
    pub fn read_upload_metadata(&mut self) -> Result<UploadMetadata, error::Error> {
        let len = self.read_len()?;
        let mut buffer = vec![0; len as usize];

        self.stream.read_exact(&mut buffer)?;
        let upload_metadata = UploadMetadata::from_bytes(&buffer);

        self.compression = upload_metadata.compression.as_ref().map(|c| c.get_algo());
        self.checksum = upload_metadata.checksum.as_ref().map(|c| c.get_algo());

        Ok(upload_metadata)
    }
}

/// Methods aimed for writing to stram
impl Link {
    pub fn write_to_stream(&mut self, source: &path::Path, relative_path: &path::Path) -> Result<(), error::Error> {
        self.write_file_metadata(relative_path)?;

        let file = std::fs::File::open(source)?;
        let mut reader = std::io::BufReader::new(file);
        let mut buffer = [0; CHUNK];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                tracing::info!("reached end of file");
                self.upstream(EOF_MARKER.as_ref())?;
                break;
            }

            self.upstream(&buffer)?;
        }

        Ok(())
    }

    /// method to send file metadata
    pub fn write_file_metadata(&mut self, relative_path: &path::Path) -> Result<(), error::Error> {
        let file_metadata = FileMetadata::new(relative_path);
        self.upstream(&file_metadata.to_bytes())
    }

    /// method to upload the chunk
    /// - Calcualtes the checksum
    /// - Sends the length of the checksum
    /// - Sends the checksum
    /// - Compresses the chunk
    /// - Sends the length of the compressed chunk
    /// - Sends the compressed chunk
    fn upstream(&mut self, buffer: &[u8]) -> Result<(), error::Error> {
        if let Some(ref algo) = self.checksum {
            let checksum = algo.generate(buffer);
            let checksum_bytes = checksum.as_bytes();
            tracing::debug!("writing checksum");
            self.write_len(checksum_bytes.len())?;
            self.stream.write_all(checksum_bytes)?;
        }

        let buffer = match self.compression {
            None => buffer.to_vec(),
            Some(ref algo) => algo.compress(buffer)?,
        };

        tracing::info!("writing buffer");
        self.write_len(buffer.len())?;
        self.stream.write_all(&buffer)?;

        Ok(())
    }

    /// method to write length to the stream
    fn write_len(&mut self, len: usize) -> Result<(), error::Error> {
        let len = len as u32;
        self.stream.write_all(len.to_be_bytes().as_ref())?;
        tracing::info!("written len: {}", len);
        Ok(())
    }

    /// method to write the download metadata to the stream
    pub fn write_download_metadata(&mut self, destination: &path::Path) -> Result<(), error::Error> {
        let download_metadata = DownloadMetadata::new(destination).with_compression(self.compression.as_ref().map(|c| c.get_type())).with_checksum(self.checksum.as_ref().map(|c| c.get_type()));
        let bytes = download_metadata.to_bytes();
        self.write_len(bytes.len())?;
        self.stream.write_all(&bytes)?;
        Ok(())
    }

    /// method to assign role to the server
    pub fn write_role(&mut self, role: Role) -> Result<(), error::Error> {
        let bytes = role.to_bytes();
        self.write_len(bytes.len())?;
        self.stream.write_all(&bytes)?;
        Ok(())
    }

    pub fn write_err_result(&mut self, msg: String) -> Result<(), error::Error> {
        let result = super::Result::Err(msg);
        let bytes = result.to_bytes();
        self.upstream(&bytes)?;
        Ok(())
    }

    pub fn write_ok_result(&mut self, count: usize) -> Result<(), error::Error> {
        let result = super::Result::Marker { count: count as u32, marker: EOF_MARKER.to_vec() };
        let bytes = result.to_bytes();
        tracing::info!("writing ok result");
        self.upstream(&bytes)?;
        Ok(())
    }

    /// raw upload
    pub fn write_upload_metadata(&mut self, count: usize, destination: &path::Path) -> Result<(), error::Error> {
        let upload_metadata = UploadMetadata::new(count as u32, destination).with_compression(self.compression.as_ref().map(|c| c.get_type())).with_checksum(self.checksum.as_ref().map(|c| c.get_type()));
        let bytes = upload_metadata.to_bytes();
        self.write_len(bytes.len())?;
        self.stream.write_all(&bytes)?;
        Ok(())

    }
}
