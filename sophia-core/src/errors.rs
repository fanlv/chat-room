use std::net;

use thiserror::Error;

// pub type Result<'a, T, E = SError<'a>> = core::result::Result<T, E>;
pub type Result<T, E = Errno> = core::result::Result<T, E>;


#[derive(Error, Debug)]
// pub enum SError<'a> {
pub enum Errno {
    #[error("error `{0}`")]
    New(String),
    #[error("connection closed")]
    ConnectionClosed,
    #[error(transparent)]
    RustlsError(#[from] rustls::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ConnectionError(#[from] quinn::ConnectionError),
    #[error(transparent)]
    WriteError(#[from] quinn::WriteError),
    #[error(transparent)]
    ReadToEndError(#[from] quinn::ReadToEndError),
    #[error(transparent)]
    ConnectError(#[from] quinn::ConnectError),
    #[error(transparent)]
    AddrParseError(#[from] net::AddrParseError),
    #[error(transparent)]
    JsonEncodeErr(#[from] serde_json::Error),
}








/*
#[derive(Error, Debug)]
pub enum GrepError {
    #[error("Glob pattern error")]
    GlobPatternError(#[from] glob::PatternError),
    #[error("Regex pattern error")]
    RegexPatternError(#[from] regex::Error),
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
}


#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DataStoreError {
    #[error("data store disconnected")]
    Disconnect(#[from] io::Error),
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
    #[error("unknown data store error")]
    Unknown,
}



#[derive(Error, Debug)]
pub enum KvError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Frame is larger than max size")]
    FrameError,
    #[error("Command is invalid: `{0}`")]
    InvalidCommand(String),
    #[error("Cannot convert value {0} to {1}")]
    ConvertError(String, &'static str),
    #[error("Cannot process command {0} with table: {1}, key: {2}. Error: {}")]
    StorageError(&'static str, String, String, String),
    #[error("Certificate parse error: error to load {0} {0}")]
    CertifcateParseError(&'static str, &'static str),

    #[error("Failed to encode protobuf message")]
    EncodeError(#[from] prost::EncodeError),
    #[error("Failed to decode protobuf message")]
    DecodeError(#[from] prost::DecodeError),
    #[error("Failed to access sled db")]
    SledError(#[from] sled::Error),
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    #[error("TLS error")]
    TlsError(#[from] tokio_rustls::rustls::TLSError),
    #[error("Yamux Connection error")]
    YamuxConnectionError(#[from] yamux::ConnectionError),
    #[error("Parse config error")]
    ConfigError(#[from] toml::de::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}
*/