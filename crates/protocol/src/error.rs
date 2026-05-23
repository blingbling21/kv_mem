use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("解析错误: {0}")]
    ParseError(String),
    #[error("UTF-8 解析错误: {0}")]
    ParseUtf8Error(#[from] std::str::Utf8Error),
    // #[error("IO 错误: {0}")]
    // IOError(#[from] std::io::Error),
    // #[error("存储错误: {0}")]
    // StorageError(#[from] storage::error::StorageError),
}

pub type ProtocolResult<T> = std::result::Result<T, ProtocolError>;
