use protocol::error::ProtocolError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("IO 错误: {0}")]
    StdError(#[from] std::io::Error),
    #[error("协议 错误: {0}")]
    ProtocolError(#[from] ProtocolError),
    #[error("UTF-8 解析错误: {0}")]
    ParseUtf8Error(#[from] std::str::Utf8Error),
    #[error("错误: {0}")]
    CliError(String),
}

pub type ClientResult<T> = std::result::Result<T, ClientError>;
