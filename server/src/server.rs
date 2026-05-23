use protocol::{command::Command, response::Response};
use storage::db::DB;

use crate::error::ServerResult;

pub struct Server {}

impl Server {
    /// 执行 cmd 并返回 Response
    pub async fn execute(cmd: Command<'_>, db: &mut DB, raw_cmd: &[u8]) -> ServerResult<Response> {
        match cmd {
            Command::Get { key } => {
                let db_guard = db.read().await;
                if let Some(val) = db_guard.get(key) {
                    Ok(Response::Value(val.clone()))
                } else {
                    Ok(Response::KeyNotExist)
                }
            }
            Command::Set { key, value } => {
                let mut db_guard = db.write().await;
                db_guard.wal_write(raw_cmd).await?; // 将原始命令写入 WAL 文件

                db_guard.set(key, value.to_vec());
                Ok(Response::Ok)
            }
            Command::Delete { key } => {
                let mut db_guard = db.write().await;
                db_guard.wal_write(raw_cmd).await?; // 将原始命令写入 WAL 文件

                if db_guard.delete(key).is_some() {
                    Ok(Response::Deleted)
                } else {
                    Ok(Response::KeyNotExist)
                }
            }
        }
    }
}
