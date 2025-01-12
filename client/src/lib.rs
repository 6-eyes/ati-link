mod conf;
mod link;

use std::net::TcpListener;

use link::Link;
use conf::fetch_conf;

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder().with_max_level(tracing::Level::TRACE).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let conf = fetch_conf()?;
    tracing::info!("starting client...\nlistening on address {}", conf.socket);

    let listener = TcpListener::bind(conf.socket).inspect_err(|e| tracing::error!("cannot connet to socket {0}. {1}", conf.socket, e))?;
    listener.set_ttl(conf.ttl.as_secs() as u32).inspect_err(|e| tracing::error!("error setting TTL {e}"))?;

    for stream in listener.incoming() {
        // don't terminate if a stream connection fails
        match stream {
            Ok(s) => {
                s.set_read_timeout(conf.read_timeout).inspect_err(|e| tracing::error!("error setting read timeout {e}"))?;
                let mut link = Link::new(s);
                link.listen()?;
            },
            Err(e) => tracing::error!("connection failed connecting to address: {e}"),
        }
    }

    Ok(())
}
