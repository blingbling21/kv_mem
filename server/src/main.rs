use std::{sync::Arc};

use protocol::command::Command;
use storage::db::{DB, DbState};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    spawn, time::{interval, Duration},
};

use crate::{error::ServerResult, server::Server};

pub mod error;
pub mod server;

#[tokio::main]
async fn main() -> ServerResult<()> {
    let wal_file_path = "wal.log";
    let snapshot_path = "snapshot.db";
    
    let db_state = DbState::recover(wal_file_path, snapshot_path).await?;

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("服务器正在监听 127.0.0.1:8080");

    let mut db_state_clone1 = Arc::clone(&db_state);
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(5));

        println!("snapshot 定时器已启动");

        loop {
            interval.tick().await;

            if let Err(e) = Server::snapshot(&mut db_state_clone1, wal_file_path, snapshot_path).await {
                eprintln!("存储snapshot过程中出现错误: {:?}", e);
            }
        }
    });

    loop {
        let (stream, _) = listener.accept().await?;

        let db_state_clone = Arc::clone(&db_state);

        spawn(async move {
            if let Err(e) = handle_client(stream, db_state_clone).await {
                eprintln!("处理客户端请求时发生错误: {}", e);
            }
        });
    }
}

async fn handle_client(mut stream: TcpStream, mut db: DB) -> ServerResult<()> {
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    if n == 0 {
        return Ok(());
    }

    let raw_cmd = &buffer[..n];
    let (cmd, _) = Command::decode(raw_cmd)?;

    let response = Server::execute(cmd, &mut db, raw_cmd).await?;

    stream.write_all(&response.encode()).await?;
    stream.flush().await?;
    Ok(())
}
