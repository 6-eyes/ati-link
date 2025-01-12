use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use commons::Error;

/// method to create configuration based on the arguments passed
pub fn fetch_conf() -> Result<Conf, Error> {
    let mut conf = Conf::default();
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let it = args.chunks(2);

    for chunk in it {
        if chunk.len() != 2 {
            return Err(Error::InvalidArgument("invalid argument detected".to_string()));
        }

        let (prop, val) = (&chunk[0], &chunk[1]);
        if prop.starts_with('-') {
            if prop == "-p" || prop == "--port" {
                conf.socket = val.parse::<SocketAddr>()?;
            }
           // add other configurations
            else {
                let err = format!("invalid property {prop}");
                tracing::error!("{err}");
                return Err(Error::InvalidArgument(err));
            }
        }
        else {
            return Err(Error::InvalidArgument(format!("invalid property {prop}")));
        }
    }

    Ok(conf)

}

pub struct Conf {
    pub socket: SocketAddr,
    pub ttl: std::time::Duration,
    pub read_timeout: Option<std::time::Duration>,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            socket: SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 9099),
            ttl: std::time::Duration::from_secs(100),
            read_timeout: Some(std::time::Duration::from_secs(10)),
        }
    }
}
