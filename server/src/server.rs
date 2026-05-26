use std::{collections::HashMap, path::Path};

use protocol::{command::Command, response::Response};
use storage::db::DB;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

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

    pub async fn snapshot(
        db: &mut DB,
        wal_log_path: &str,
        snapshot_path: &str,
    ) -> ServerResult<()> {
        let wal_log_old_path = format!("{}.old", wal_log_path);
        let snapshot_tmp_path = format!("{}.tmp", snapshot_path);

        let map_clone = {
            let mut db_guard = db.write().await;
            let wal_size = db_guard.get_wal_size();
            if wal_size < 20 * 1024 * 1024 {
                return Ok(());
            }

            if Path::new(wal_log_path).exists() {
                fs::rename(&wal_log_path, &wal_log_old_path).await?;
            }

            let wal_file = File::options()
                .create(true)
                .append(true)
                .open(wal_log_path)
                .await?;
            db_guard.reset_wal(wal_file);
            let hashmap_clone = db_guard.get_db_data_clone();
            hashmap_clone
        };

        let snapshot_path_clone = snapshot_path.to_string();

        tokio::spawn(async move {
            if let Err(e) = save_to_tmp_file(&snapshot_tmp_path, &map_clone).await {
                eprintln!("后台snapshot写入失败: {:?}", e);
                return;
            }

            if let Err(e) = fs::rename(snapshot_tmp_path, snapshot_path_clone).await {
                eprintln!("后台替换snapshot文件失败: {:?}", e);
                return;
            }

            if let Err(e) = fs::remove_file(wal_log_old_path).await {
                eprintln!("后台删除wal.log.old文件失败: {:?}", e);
            } else {
                println!("后台snapshot保存成功！")
            }
        });

        Ok(())
    }
}

/// 辅助方法，将map内存数据按Command::Set指令写入临时文件
async fn save_to_tmp_file(file_path: &str, map: &HashMap<String, Vec<u8>>) -> ServerResult<()> {
    let mut tmp_file = File::create(file_path).await?;

    for (key, value) in map {
        let cmd = Command::Set { key, value };
        tmp_file.write_all(&cmd.encode()).await?;
    }
    tmp_file.flush().await?;

    Ok(())
}
