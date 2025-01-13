mod conf;

use std::net::TcpListener;
use conf::fetch_conf;

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let conf = fetch_conf()?;
    let tracing_level = match conf.debug {
        true => tracing::Level::TRACE,
        false => tracing::Level::INFO,
    };

    let subscriber = tracing_subscriber::FmtSubscriber::builder().with_max_level(tracing_level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tracing::info!("starting client...\nlistening on address {}", conf.socket);

    let listener = TcpListener::bind(conf.socket).inspect_err(|e| tracing::error!("cannot connet to socket {0}. {1}", conf.socket, e))?;
    listener.set_ttl(conf.ttl.as_secs() as u32).inspect_err(|e| tracing::error!("error setting TTL {e}"))?;

    for stream in listener.incoming() {
        // don't terminate if a stream connection fails
        match stream {
            Ok(s) => {
                s.set_read_timeout(conf.read_timeout).inspect_err(|e| tracing::error!("error setting read timeout {e}"))?;
                let link = commons::connection::Link::new(s);
                let _ = listen(link);
            },
            Err(e) => tracing::error!("connection failed connecting to address: {e}"),
        }
    }

    Ok(())
}

/// use the created [`Link`](commons::connection::Link) to listen to the stream
fn listen(mut link: commons::connection::Link) -> Result<(), commons::error::Error> {
    match link.read_role()? {
        commons::Role::Source => {
            let download_metadata = link.read_download_metadata()?;
            let path = download_metadata.destination;

            if path.is_dir() {
                tracing::info!("path is a directory");
                let all_files = commons::get_recursive_paths(&path);
                let len = all_files.len();
                link.write_ok_result(len)?;

                for file in all_files {
                    match file.strip_prefix(&path) {
                        Ok(p) => link.write_to_stream(&file, &std::path::PathBuf::from(p))?,
                        Err(e) => return Err(commons::error::Error::InvalidRequest(format!("{} is not a relative path of {}, {e}", file.to_str().unwrap(), path.to_str().unwrap()))),
                    }
                }
                tracing::info!("{} files uploaded", len);
                
            }
            else if path.is_file() {
                tracing::info!("path is a file");
                link.write_ok_result(1)?;

                link.write_to_stream(&path, &std::path::PathBuf::from(path.file_name().unwrap()))?;

                tracing::info!("1 file uploaded");
            }
            else {
                tracing::error!("invalid file path");
                link.write_err_result(format!("Path {:?} invalid", path))?;
            };
        },
        commons::Role::Sink => {
            let metadata = link.read_upload_metadata()?;
            tracing::debug!("received upload metadata: {:?}", metadata);

            for _ in 0..metadata.count {
                link.read_from_stream(&metadata.destination, &metadata.eof_marker)?;
            }

            tracing::info!("{} files received", metadata.count);
        },
    };

    Ok(())
}
