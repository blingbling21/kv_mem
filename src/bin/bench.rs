// use std::time::Instant;

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, spawn, time::{Instant, sleep}};

#[tokio::main]
async fn main() {
    // 1. 设置并发客户端数量（建议先从 200 开始，逐步往上加）
    let concurrency = 10000; 
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
            let key = format!("key_{:04}", i);
            let _val = "val_test";
            
            // 拼接二进制数据包
            let mut payload = vec![1]; // Cmd 2 = SET
            payload.extend_from_slice(&(key.len() as u16).to_be_bytes()); // Key 长度 (u16)
            payload.extend_from_slice(key.as_bytes());                   // Key 内容
            // payload.extend_from_slice(&(val.len() as u16).to_be_bytes()); // Value 长度 (u16)
            // payload.extend_from_slice(val.as_bytes());                   // Value 内容

            // 发送数据
            if stream.write_all(&payload).await.is_err() {
                return;
            }

            // 读取响应
            let mut response = [0; 13]; // 期望读到 "KEY_NOT_EXIST" 十三个字节
            if let Ok(n) = stream.read(&mut response).await {
                if n == 13 && &response[..13] == b"KEY_NOT_EXIST" {
                    // 解析成功，保持连接不关闭，模拟高并发挂起状态
                    // 故意睡眠 1 秒，让所有连接同时保持在线
                    // sleep(std::time::Duration::from_secs(1)).await;
                }  else {
                    eprintln!("客户端 {} 收到异常响应: {:?}", i, String::from_utf8_lossy(&response[..n]));
                }
            }
        });
        handles.push(handle);

        if i % 200 == 0 {
          sleep(std::time::Duration::from_millis(20)).await; // 小间隔，逐步建立连接，避免瞬间过载
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