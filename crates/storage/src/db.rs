use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
};

use protocol::command::Command;
use crate::error::{StorageError::ParseError, StorageResult};

pub type DB = Arc<RwLock<DbState>>;

pub struct DbState {
    map: HashMap<String, Vec<u8>>,
    wal_file: File,
}

impl DbState {
    pub async fn recover(file_path: &str) -> StorageResult<DB> {
        let mut temp_map = HashMap::new();

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
                        temp_map.insert(key.to_string(), value.to_vec());
                    }
                    Command::Delete { key } => {
                        temp_map.remove(key);
                    }
                    _ => {
                        return Err(ParseError("解析失败，未知的Command类型。".to_string()));
                    }
                }
                cursor += protocol_len; // 移动到下一个命令
            }

            println!("数据恢复完成，成功恢复了 {} 条记录。", temp_map.len());
        } else {
            println!("未检测到 WAL 文件，初始化空白数据库。");
        }

        let wal_file = File::options()
            .create(true)
            .append(true)
            .open(file_path)
            .await?;

        Ok(Arc::new(RwLock::new(DbState {
            map: temp_map,
            wal_file,
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
        Ok(())
    }
}
