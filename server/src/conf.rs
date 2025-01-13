use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use commons::error::Error;

/// method to create configuration based on the arguments passed
pub fn fetch_conf() -> Result<Conf, Error> {
    let mut conf = Conf::default();
    let mut args = std::env::args().skip(1);

    while let Some(s) = args.next() {
        match s.as_str() {
            "-p" | "--port" => match args.next() {
                None => return Err(Error::invalid_argument("no port value supplied")),
                Some(v) => conf.socket = v.parse::<SocketAddr>()?,
            },
            "-d" | "--debug" => conf.debug = true,
            _ => return Err(Error::InvalidArgument(format!("invalid property {s}"))), 
        }
    }

    Ok(conf)

}

pub struct Conf {
    pub socket: SocketAddr,
    pub ttl: std::time::Duration,
    pub read_timeout: Option<std::time::Duration>,
    pub debug: bool,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            socket: SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 9099),
            ttl: std::time::Duration::from_secs(100),
            read_timeout: Some(std::time::Duration::from_secs(10)),
            debug: false,
        }
    }
}
