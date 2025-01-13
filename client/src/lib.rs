mod conf;

use std::{net::TcpStream, time::Instant};

use conf::fetch_conf;

/// method to load the configuration and initialize the link
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder().with_max_level(tracing::Level::TRACE).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut conf = fetch_conf()?;

    let socket = conf.socket()?;
    let source = conf.source()?;
    let sink = conf.sink()?;
    let role = conf.role()?;

    let stream = TcpStream::connect(socket).inspect_err(|e| eprintln!("cannot connect to receiver {0}. {1}", socket, e))?;
    stream.set_write_timeout(conf.write_timeout).inspect_err(|e| eprintln!("error setting timeout {e}"))?;

    let instant = Instant::now();
    let mut link = commons::connection::Link::new(stream).with_checksum(conf.checksum).with_compression(conf.compression);
    link.write_role(role)?;

    match role {
        commons::Role::Source => {
            link.write_download_metadata(&source)?;
            tracing::debug!("written download metadata");
            match link.read_result()? {
                commons::Result::Err(s) => {
                    eprintln!("error in transfer: {s}");
                    return error(commons::error::Error::DownloadError(s));
                },
                commons::Result::Marker { count, marker } => {
                    for _ in 0..count {
                        link.read_from_stream(&sink, &marker)?;
                    }
                    println!("{count} files read");
                },
            }
        },
        commons::Role::Sink => {
            if source.is_dir() {
                tracing::debug!("source is a directory!");
                let all_files = commons::get_recursive_paths(&source);
                let len = all_files.len();
                link.write_upload_metadata(all_files.len(), &sink)?;

                for file in all_files {
                    match file.strip_prefix(&source) {
                        Ok(relative_path) => {
                            link.write_to_stream(&file, &std::path::PathBuf::from(relative_path))?;
                        },
                        Err(e) => return error(commons::error::Error::InvalidRequest(format!("{} is not a relative path of {}, {e}", file.to_str().unwrap(), source.to_str().unwrap()))),
                    }
                }

                println!("{} files uploaded", len);
            }
            else if source.is_file() {
                println!("source is a file");
                let rel = std::path::Path::new(source.file_name().unwrap());
                link.write_upload_metadata(1, &sink)?;
                link.write_to_stream(&source, rel)?;
            }
            else if source.is_symlink() {
               eprintln!("symlinks not supported");
            }
        },
    }

    let elapsed = instant.elapsed();
    println!("Time taken: {:?}", elapsed);

    Ok(())
}

fn error(error: commons::error::Error) -> Result<(), Box<dyn std::error::Error>> {
    Err(Box::new(error))
}
