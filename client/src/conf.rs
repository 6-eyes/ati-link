use std::{net::SocketAddr, path, time::Duration};
use commons::error::Error;

pub fn fetch_conf() -> Result<Conf, Error> {
    let mut conf = Conf::default();
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let mut it = args.into_iter().peekable();

    while let Some(prop) = it.next() {
        match prop.as_str() {
            "--source" | "-s" => match it.next() {
                None => {
                    let err = "No value for source path provided";
                    eprintln!("{err}");
                    return Err(Error::invalid_argument(err));
                },
                Some(p) => {
                    match p.split_once('@') {
                        Some((address, path)) => {
                            conf.set_socket(address, commons::Role::Source, true)?;
                            conf.source_path(path, true)?
                        },
                        None => conf.source_path(&p, true)?,
                    }
                },
            },
            "--destination" | "-d" => match it.next() {
                None => {
                    let err = "No value for destination path provided";
                    eprintln!("{err}");
                    return Err(Error::invalid_argument(err));
                },
                Some(p) => match p.split_once('@') {
                    None => conf.sink_path(&p, true)?,
                    Some((address, path)) => {
                        conf.set_socket(address, commons::Role::Sink, true)?;
                        conf.sink_path(path, true)?
                    }
                },
            },
            "--compression" | "-co" => match it.next() {
                None => {
                    let err = "No value provided for compression";
                    eprintln!("{err}");
                    return Err(Error::invalid_argument(err));
                },
                Some(co) => if let Err(e) = conf.compression(&co) {
                    eprintln!("{e}");
                    return Err(e);
                },
            },
            "--checksum" | "-ch" => match it.next() {
                None => {
                    let err = "No value provided for checksum";
                    eprintln!("{err}");
                    return Err(Error::invalid_argument(err));
                },
                Some(co) => if let Err(e) = conf.checksum(&co) {
                    eprintln!("{e}");
                    return Err(e);
                },
            },
            // add more configurations here
            _ => {
                let err = format!("only one source and destination are allowed. Reading {prop}");
                eprintln!("{}", err);
                return Err(Error::InvalidArgument(err));
            },
        };
    }

    Ok(conf)
}

/// Server configuration
pub struct Conf {
    source: Option<path::PathBuf>,
    sink: Option<path::PathBuf>,
    role: Option<commons::Role>,
    socket: Option<SocketAddr>,
    pub write_timeout: Option<std::time::Duration>,
    pub compression: Option<Box<dyn commons::compression::Compression>>,
    pub checksum: Option<Box<dyn commons::checksum::Checksum>>,
}

impl Default for Conf {
    fn default() -> Self {
        let settings = file_config::Settings::load();
        let mut config = Self {
            source: None,
            sink: None,
            role: None,
            socket: None,
            write_timeout: settings.write_timeout.map(Duration::from_secs),
            compression: None,
            checksum: None,
        };

        if let Some(source) = settings.source {
            if let Some((address, path)) = source.split_once('@') {
                config.set_socket(address, commons::Role::Source, false).unwrap();
                config.source_path(path, false).unwrap();
            }
            else {
                config.source_path(&source, false).unwrap();
            }
        }

        if let Some(sink) = settings.sink {
            if let Some((address, path)) = sink.split_once('@') {
                config.set_socket(address, commons::Role::Sink, false).unwrap();
                config.sink_path(path, false).unwrap();
            }
            else {
                config.sink_path(&sink, false).unwrap();
            }
        }

        if let Some(c) = settings.compression {
            let _ = config.compression(&c);
        }

        if let Some(c) = settings.checksum {
            let _ = config.checksum(&c);
        }
        config
    }
}

impl Conf {
    pub fn source(&mut self) -> Result<path::PathBuf, Error> {
        self.source.take().ok_or(Error::invalid_argument("no source path defined"))
    }

    pub fn sink(&mut self) -> Result<path::PathBuf, Error> {
        self.sink.take().ok_or(Error::invalid_argument("no sink path defined"))
    }

    pub fn socket(&self) -> Result<SocketAddr, Error> {
        self.socket.ok_or(Error::invalid_argument("no socket path defined"))
    }

    pub fn role(&self) -> Result<commons::Role, Error> {
        self.role.ok_or(Error::invalid_argument("server role not determined"))
    }

    /// Method to add source path
    /// validate the source path is it exists on localhost
    /// paths with sockets defined with them are not validated
    fn source_path(&mut self, path: &str, cli: bool) -> Result<(), Error> {
        if !cli || self.sink.is_none() {
            let path = path::PathBuf::from(path);
            self.source = Some(path);
            Ok(())
        }
        else {
            Err(Error::InvalidArgument("only one source path is allowed".into()))
        }
    }

    /// Method to add sink path
    /// only one sink path is allowed
    /// paths with sockets defined with them are not validated
    fn sink_path(&mut self, path: &str, cli: bool) -> Result<(), Error> {
        if !cli || self.sink.is_none() {
            let path = path::PathBuf::from(path);
            self.sink = Some(path);
            Ok(())
        }
        else {
            Err(Error::InvalidArgument("only one sink path is allowed".into()))
        }
    }

    /// Method to add socket address
    /// only one socket address is allowed, either with source or with sink
    fn set_socket(&mut self, socket: &str, role: commons::Role, cli: bool) -> Result<(), Error> {
        if !cli || self.socket.is_none() {
            let s = socket.parse::<SocketAddr>()?;
            self.socket = Some(s);
            self.role = Some(role);
            Ok(())
        }
        else {
            Err(Error::InvalidArgument("only one socket address is allowed".into()))
        }
    }

    fn compression(&mut self, compression: &str) -> Result<(), Error> {
        self.compression = match compression {
            "Zlib" => Some(Box::new(commons::compression::Zlib)),
            "GZip" => Some(Box::new(commons::compression::GZip)),
            _ => return Err(Error::InvalidArgument(format!("Invalid compression type {compression}"))),
        };

        Ok(())
    }

    fn checksum(&mut self, checksum: &str) -> Result<(), Error> {
        self.checksum = match checksum {
            "Sha256" => Some(Box::new(commons::checksum::Sha256)),
            "Md5" => Some(Box::new(commons::checksum::Md5)),
            _ => return Err(Error::InvalidArgument(format!("Invalid checksum type {checksum}"))),
        };

        Ok(())
    }
}

mod file_config {
    const FILE_NAME: &str = "atilink-conf.toml";

    #[derive(Debug, Default)]
    pub struct Settings {
        pub source: Option<String>,
        pub sink: Option<String>,
        pub compression: Option<String>,
        pub checksum: Option<String>,
        pub chunk_bytes: Option<u64>,
        pub write_timeout: Option<u64>,
    }

    impl Settings {
        pub fn load() -> Self {
            let mut settings = Self::default();
            if let Ok(config_str) = std::fs::read_to_string(FILE_NAME) {
                let value = config_str.parse::<toml::Table>().unwrap();
                if let Some(value) = value.get("settings") {
                    settings.source = value.get("source").and_then(toml::Value::as_str).map(str::to_string);
                    settings.sink = value.get("sink").and_then(toml::Value::as_str).map(str::to_string);
                    settings.compression = value.get("compression").and_then(toml::Value::as_str).map(str::to_string);
                    settings.checksum = value.get("checksum").and_then(toml::Value::as_str).map(str::to_string);
                    settings.chunk_bytes = value.get("chunk-bytes").and_then(|v| v.as_integer()).map(|v| v as u64);
                    settings.write_timeout = value.get("write-timeout-sec").and_then(|v| v.as_integer()).map(|v| v as u64);
                }
            }

            settings
        }
    }
}
