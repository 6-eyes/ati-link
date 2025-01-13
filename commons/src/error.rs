use std::net::AddrParseError;

#[derive(Debug)]
pub enum Error {
    InvalidAddress,
    InvalidArgument(String),
    IO(std::io::Error),
    Deserialize,
    ConnectionFailed,
    IntegrityError(String),
    DownloadError(String),
    InvalidRequest(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAddress => write!(f, "invalid address input"),
            Self::InvalidArgument(s) => write!(f, "{s}"),
            Self::IO(e) => write!(f, "io error {e}"),
            Self::Deserialize => write!(f, "unable to parse string"),
            Self::ConnectionFailed => write!(f, "unable to connect"),
            Self::IntegrityError(s) => write!(f, "{s}"),
            Self::DownloadError(s) => write!(f, "{s}"),
            Self::InvalidRequest(s) => write!(f, "{s}"),
        }
    }
}

impl From<AddrParseError> for Error {
    fn from(value: AddrParseError) -> Self {
        eprintln!("{value}");
        Self::InvalidAddress
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(_value: std::num::ParseIntError) -> Self {
        Self::Deserialize
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl Error {
    pub fn integrity_error(s: &str) -> Self {
        Self::IntegrityError(s.to_string())
    }

    pub fn invalid_argument(s: &str) -> Self {
        Self::InvalidArgument(s.to_string())
    }

    pub fn download_error(s: &str) -> Self {
        Self::DownloadError(s.to_string())
    }

    pub fn invalid_request(s: &str) -> Self {
        Self::InvalidRequest(s.to_string())
    }
}
