use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("解析错误: {0}")]
    ParseError(String),
    #[error("UTF-8 解析错误: {0}")]
    ParseUtf8Error(#[from] std::str::Utf8Error),
    #[error("IO 错误: {0}")]
    IOError(#[from] std::io::Error),
    #[error("协议错误: {0}")]
    CommandError(#[from] protocol::error::ProtocolError),
}

pub type StorageResult<T> = std::result::Result<T, StorageError>;
