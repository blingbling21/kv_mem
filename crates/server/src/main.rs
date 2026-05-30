use std::sync::Arc;

use protocol::{command::Command, response::Response};
use storage::db::{DB, DbState};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::watch::{self, Receiver},
    task::JoinSet,
    time::{Duration, interval},
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

            if let Err(e) =
                Server::snapshot(&mut db_state_clone1, wal_file_path, snapshot_path).await
            {
                eprintln!("存储snapshot过程中出现错误: {:?}", e);
            }
        }
    });

    let mut active_connections = JoinSet::new();

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    loop {
        tokio::select! {
            accept = listener.accept() => {
                match accept {
                    Ok((stream, _)) => {
                        let db_state_clone = Arc::clone(&db_state);
                        let mut shutdown_rx_clone = shutdown_rx.clone();
                        active_connections.spawn(async move {
                            if let Err(e) = handle_client(stream, db_state_clone, &mut shutdown_rx_clone).await {
                                eprintln!("处理客户端请求时发生错误: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("连接失败：{:?}", e)
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\n接收到 Ctrl + C 信号，正在停止接收新连接...");
                shutdown_tx.send(true)?;
                break;
            }
        }
    }

    while let Some(_) = active_connections.join_next().await {}
    println!("所有连接已安全关闭。");

    let mut db_guard = db_state.write().await;
    db_guard.write_sync_all().await?;

    println!("服务器已停止。");

    Ok(())
}

async fn handle_client(mut stream: TcpStream, mut db: DB, shutdown_rx: &mut Receiver<bool>) -> ServerResult<()> {
    let mut buffer = [0; 1024];
    loop {
        tokio::select! {
            res = handle_sigle_client(&mut stream, &mut db, &mut buffer) => {
                match res {
                    Ok(_) => {},
                    Err(e) => {
                        return Err(e)
                    },
                }
            }
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    let response = Response::Shutdown;
                    stream.write_all(&response.encode()).await?;
                    stream.flush().await?;
                    println!("🔌 已主动断开一个客户端（因服务端正在平滑关闭）");
                    return Ok(())
                }
            }
        }
    }
}

async fn handle_sigle_client(stream: &mut TcpStream, db: &mut DB, buffer: &mut [u8; 1024]) -> ServerResult<()> {
    let n = stream.read(buffer).await?;
    if n == 0 {
        return Ok(());
    }

    let raw_cmd = &buffer[..n];
    let (cmd, _) = Command::decode(raw_cmd)?;

    let response = Server::execute(cmd, db, raw_cmd).await?;

    stream.write_all(&response.encode()).await?;
    stream.flush().await?;
    Ok(())
}
