use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread::spawn,
};

use crate::error::AppResult;

pub mod error;

type Db = Arc<Mutex<HashMap<String, Vec<u8>>>>;

fn main() -> AppResult<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("服务器正在监听 127.0.0.1:8080");
    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("接受连接时发生错误: {}", e);
                continue;
            }
        };

        let db_clone = Arc::clone(&db);

        spawn(|| {
            if let Err(e) = handle_client(stream, db_clone) {
                eprintln!("处理客户端请求时发生错误: {}", e);
            }
        });
    }

    Ok(())
}

#[derive(Debug)]
pub enum Command<'a> {
    Get { key: &'a str },
    Set { key: &'a str, value: &'a [u8] },
    Delete { key: &'a str },
}

pub fn parse_command<'a>(input: &'a [u8]) -> AppResult<Command<'a>> {
    if input.len() < 3 {
        return Err(error::Error::ParseError(
            "解析失败，字节长度不足。".to_string(),
        ));
    }
    let command_code = input[0];
    let array = [input[1], input[2]];
    let key_len = u16::from_be_bytes(array) as usize;

    if input.len() < 3 + key_len {
        return Err(error::Error::ParseError(
            "解析失败，字节长度不足以包含键。".to_string(),
        ));
    }

    let key = str::from_utf8(&input[3..3 + key_len])?;

    match command_code {
        1 => Ok(Command::Get { key }),
        2 => {
            if input.len() < 3 + key_len + 2 {
                return Err(error::Error::ParseError(
                    "解析失败，字节长度不足".to_string(),
                ));
            }
            let value_len = u16::from_be_bytes([input[3 + key_len], input[4 + key_len]]) as usize;
            if input.len() < 3 + key_len + 2 + value_len {
                return Err(error::Error::ParseError(
                    "解析失败，字节长度不足以包含值。".to_string(),
                ));
            }
            let value = &input[5 + key_len..5 + key_len + value_len];
            return Ok(Command::Set { key, value });
        }
        3 => Ok(Command::Delete { key }),
        _ => Err(error::Error::ParseError(
            "解析失败，未知的Command类型。".to_string(),
        )),
    }
}

fn handle_client(mut stream: TcpStream, db: Db) -> AppResult<()> {
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer)?;
    if n == 0 {
        return Ok(());
    }
    let cmd = parse_command(&buffer[..n])?;

    let mut db_guard = db.lock().unwrap();
    match cmd {
        Command::Get { key } => {
            if let Some(val) = db_guard.get(key) {
                stream.write_all(val)?;
                stream.write_all(b"\n")?;
            } else {
                stream.write_all(b"KEY_NOT_EXIST\n")?;
            }
        }
        Command::Set { key, value } => {
            db_guard.insert(key.to_string(), value.to_vec());
            stream.write_all(b"OK\n")?;
        }
        Command::Delete { key } => {
            if db_guard.remove(key).is_some() {
                stream.write_all(b"DELETED\n")?;
            } else {
                stream.write_all(b"KEY_NOT_EXIST\n")?;
            }
        }
    }
    stream.flush()?;
    Ok(())
}
