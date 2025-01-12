use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use commons::Error;

pub fn fetch_conf() -> Result<Conf, Error> {
    let mut conf = Conf::default();
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let mut it = args.into_iter().peekable();

    while let Some(prop) = it.next() {
        if prop.starts_with('-') {
            if let Some(val) = it.next() {
                if prop == "-p" || prop == "--port" {
                    conf.socket = val.parse::<SocketAddr>()?;
                }
                // add other configurations
                else {
                    let err = format!("invalid property {prop}");
                    eprintln!("{err}");
                    return Err(Error::InvalidArgument(err));
                }
            }
            else {
                let err = format!("no value found for property {prop}");
                eprintln!("{err}");
                return Err(Error::InvalidArgument(err));
            }
        }
        else {
            println!("reading path {prop}");
            match it.peek() {
                None => conf.add_sink_path(&prop),
                Some(_) => conf.add_source_path(&prop),
            }?
        }
    }

    Ok(conf)
}

/// Server configuration
#[derive(Debug)]
pub struct Conf {
    pub socket: SocketAddr,
    pub source: Vec<std::path::PathBuf>,
    pub sink: Option<std::path::PathBuf>,
    pub write_timeout: Option<std::time::Duration>,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            socket: SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 9099),
            source: Vec::new(),
            sink: None,
            write_timeout: None,
        }
    }
}

impl Conf {
    /// Method to add source path to the list
    fn add_source_path(&mut self, path: &str) -> Result<(), Error> {
        match std::fs::exists(path)? {
            true => {
                self.source.push(std::path::PathBuf::from(path));
                Ok(())
            },
            false => Err(Error::InvalidArgument(format!("invalid path {path}"))),
        }
    }

    /// Method to add sink path
    /// Throws error if a sink path already exists
    fn add_sink_path(&mut self, path: &str) -> Result<(), Error> {
        match self.sink {
            None => {
                self.sink = Some(std::path::PathBuf::from(path));
                Ok(())
            },
            Some(_) => Err(Error::InvalidArgument("only one sink path is allowed".into()))
        }
    }
}
