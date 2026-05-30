use protocol::{command::Command, response::Response};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, spawn, time::{Instant, sleep}};

#[tokio::main]
async fn main() {
    // 1. 设置并发客户端数量（建议先从 200 开始，逐步往上加）
    let concurrency = 100000; 
    let mut handles = vec![];
    
    println!("🚀 开始建立 {} 个并发连接...", concurrency);
    let start = Instant::now();

    for i in 0..concurrency {
        let handle = spawn(async move {
            // 与服务端建立 TCP 连接
            let mut stream = match TcpStream::connect("127.0.0.1:8080").await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("客户端 {} 连接失败: {:?}", i, e);
                    return;
                }
            };
            
            // 构造自定义二进制协议数据
            // 我们为每个线程分配不同的 Key（如 key_0001, key_0002），模拟真实的业务场景
            let key = format!("key_{:06}", i);
            let val =  format!("val_iiii_{:06}", i);
            let cmd = Command::Get { key: &key }; // 这里我们先测试 Get 命令，后续可以改成 Set 或 Delete 来测试不同的命令类型
            // let cmd = Command::Set { key: &key, value: val.as_bytes() }; // 这里我们先测试 Get 命令，后续可以改成 Set 或 Delete 来测试不同的命令类型
            
            // 拼接二进制数据包
            let payload = cmd.encode();

            // 发送数据
            if stream.write_all(&payload).await.is_err() {
                return;
            }

            // 读取响应
            let mut response = [0; 13]; // 期望读到 "KEY_NOT_EXIST" 十三个字节
            if let Ok(n) = stream.read(&mut response).await {
                if let Some(res) = Response::decode(&response[..n]) {
                    match res {
                        Response::Ok => {},
                        Response::KeyNotExist => {},
                        Response::Deleted => {},
                        Response::Value(items) => {},
                        Response::Error(err_msg) => eprintln!("客户端 {} 收到错误响应: {}", i, err_msg),
                        Response::Shutdown => {},
                    }
                }
            }
        });
        handles.push(handle);

        if i % 200 == 0 {
          sleep(std::time::Duration::from_millis(50)).await; // 小间隔，逐步建立连接，避免瞬间过载
        }
    }

    // 等待所有压测线程结束
    for handle in handles {
        let _ = handle.await;
    }

    let duration = start.elapsed();
    println!("✨ 所有并发客户端测试完成！");
    println!("📊 并发量: {}，总耗时: {:?}", concurrency, duration);
}