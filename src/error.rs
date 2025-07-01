use crate::wg::config::ParseError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("parse error: {0}")]
    ParseError(#[from] ParseError),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("wg cmd fail: {0:?}")]
    WgCommandFail(Option<i32>),

    #[error("irc error: {0}")]
    IrcError(#[from] irc::error::Error),

    #[error("encode error: {0}")]
    EncodeError(#[from] bincode::error::EncodeError),

    #[error("base64 decode error: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),

    #[error("decode error: {0}")]
    DecodeError(#[from] bincode::error::DecodeError),

    #[error("stun error: {0}")]
    StunError(#[from] stunclient::Error),
}
