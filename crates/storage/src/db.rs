use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
};

use crate::error::{StorageError::ParseError, StorageResult};
use protocol::command::Command;

pub type DB = Arc<RwLock<DbState>>;

pub struct DbState {
    map: HashMap<String, Vec<u8>>,
    wal_file: File,
    wal_size: usize,
}

impl DbState {
    pub async fn recover(wal_log_path: &str, snapshot_path: &str) -> StorageResult<DB> {
        let snapshot_tmp_path = format!("{}.tmp", snapshot_path);
        if Path::new(&snapshot_tmp_path).exists() {
            fs::remove_file(&snapshot_tmp_path).await?;
        }

        let mut temp_map = HashMap::new();

        if Path::new(&snapshot_path).exists() {
            DbState::replay_file(&snapshot_path, &mut temp_map).await?;
        }

        let wal_log_path_old = format!("{}old", wal_log_path);
        if Path::new(&wal_log_path_old).exists() {
            DbState::replay_file(&wal_log_path_old, &mut temp_map).await?;
        }

        let mut wal_size = 0;
        if Path::new(&wal_log_path).exists() {
            wal_size = DbState::replay_file(&wal_log_path, &mut temp_map).await?;
        }


        let wal_file = File::options()
            .create(true)
            .append(true)
            .open(wal_log_path)
            .await?;

        Ok(Arc::new(RwLock::new(DbState {
            map: temp_map,
            wal_file,
            wal_size,
        })))
    }

    /// 获取内存数据库键对应的值
    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.map.get(key)
    }

    /// 设置内存数据库键值对
    pub fn set(&mut self, key: &str, value: Vec<u8>) {
        self.map.insert(key.to_string(), value);
    }

    /// 删除内存数据库中的键
    pub fn delete(&mut self, key: &str) -> Option<Vec<u8>> {
        self.map.remove(key)
    }

    /// 预写日志记录操作
    pub async fn wal_write(&mut self, buffer: &[u8]) -> StorageResult<()> {
        self.wal_file.write_all(buffer).await?;
        self.wal_file.flush().await?;
        self.wal_size += buffer.len();
        Ok(())
    }

    /// 获取wal_size
    pub fn get_wal_size(&self) -> usize {
        self.wal_size
    }

    /// 获取db数据的拷贝
    pub fn get_db_data_clone(&self) -> HashMap<String, Vec<u8>> {
        self.map.clone()
    }

    /// 重置wal_file和wal_size
    pub fn reset_wal(&mut self, wal_file: File) {
        self.wal_file = wal_file;
        self.wal_size = 0;
    }

    /// 辅助方法，用于重放文件中的 Command 指令，并将数据写入内存数据中
    async fn replay_file(file_path: &str, map: &mut HashMap<String, Vec<u8>>) -> StorageResult<usize> {
        // 如果 WAL 文件存在，读取并恢复数据
        if Path::new(file_path).exists() {
            let mut file = File::open(file_path).await?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).await?;

            let mut cursor = 0;

            while cursor < buffer.len() {
                let (protocol, protocol_len) = Command::decode(&buffer[cursor..])?;
                match protocol {
                    Command::Set { key, value } => {
                        map.insert(key.to_string(), value.to_vec());
                    }
                    Command::Delete { key } => {
                        map.remove(key);
                    }
                    _ => {
                        return Err(ParseError("解析失败，未知的Command类型。".to_string()));
                    }
                }
                cursor += protocol_len; // 移动到下一个命令
            }

            println!("数据恢复完成，成功恢复了 {} 条记录。", map.len());
            Ok(buffer.len())
        } else {
            println!("未检测到 WAL 文件，初始化空白数据库。");
            Ok(0)
        }
        // Ok(0)
    }
}
