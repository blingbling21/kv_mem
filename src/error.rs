use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("解析错误: {0}")]
    ParseError(String),
    #[error("UTF-8 解析错误: {0}")]
    ParseUtf8Error(#[from] std::str::Utf8Error),
    #[error("IO 错误: {0}")]
    IOError(#[from] std::io::Error),
    // #[error("锁定数据库时发生错误: {0}")]
    // LockError(#[from] std::sync::PoisonError<std::sync::MutexGuard<'_, std::collections::HashMap<String, Vec<u8>>>>),
}

pub type AppResult<T> = std::result::Result<T, Error>;
