use tokio::net::TcpStream;

use crate::{client::Client, error::ClientResult};

mod error;
mod client;

#[tokio::main]
async fn main() -> ClientResult<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("已连接服务器: 127.0.0.1:8080");

    let cli = Client::new();
    cli.start(&mut stream).await?;

    Ok(())
}
