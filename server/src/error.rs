use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("存储错误: {0}")]
    StorageError(#[from] storage::error::StorageError),
    #[error("协议错误: {0}")]
    ProtocolError(#[from] protocol::error::ProtocolError),
    #[error("UTF-8 解析错误: {0}")]
    ParseUtf8Error(#[from] std::str::Utf8Error),
    #[error("IO 错误: {0}")]
    IOError(#[from] std::io::Error),
}

pub type ServerResult<T> = std::result::Result<T, ServerError>;
