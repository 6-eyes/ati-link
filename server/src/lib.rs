mod conf;
mod link;

use std::net::TcpStream;

use conf::fetch_conf;
use link::Link;

/// method to load the configuration and initialize the link
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, from server!");
    let conf = fetch_conf()?;

    let stream = TcpStream::connect(conf.socket).inspect_err(|e| eprintln!("cannot connect to receiver {0}. {1}", conf.socket, e))?;
    stream.set_write_timeout(conf.write_timeout).inspect_err(|e| eprintln!("error setting timeout {e}"))?;

    let mut link = Link::new(stream);
    link.send_file(std::path::PathBuf::from("server/data/Programming Assignment - RSE.pdf"), std::path::PathBuf::from("client/data"))?;
    Ok(())
}
