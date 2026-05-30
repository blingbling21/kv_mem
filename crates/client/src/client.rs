use std::io::Write;

use protocol::{command::Command, response::Response};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, stdin},
    net::TcpStream, select,
};

use crate::error::{ClientError, ClientResult};

pub struct Client {}

impl Client {
    pub fn new () -> Self {
        return Self {};
    }
    /**
     * 启动cli循环
     */
    pub async fn start(&self, stream: &mut TcpStream) -> ClientResult<()> {
        // println!("");

        let mut reader = BufReader::new(stdin());
        let mut detect_buf = [0u8; 10];
        loop {
            print!("kv-cli >");
            let mut input = String::new();
            std::io::stdout().flush()?;

            select! {
                _ =  reader.read_line(&mut input) => {
                    match self.user_input(&input, stream).await {
                        Ok(flag) => {
                            if !flag {
                                return Ok(())
                            }
                        },
                        Err(e) => {
                            return Err(e)
                        },
                    }
                }
                r = stream.read(&mut detect_buf) => {
                    let n = r?;
                    if n == 0 {
                        println!("\n服务器已关闭连接。");
                        return Ok(());
                    }
                    if let Some(Response::Shutdown) = Response::decode(&detect_buf[..n]) {
                        println!("\n服务器已关闭连接。");
                        return Ok(());
                    }
                }
            }
        }
    }

    /**
     * 解析cli输入的字符串
     */
    pub fn input_decode(&self, cmd_str: &str) -> ClientResult<Vec<u8>> {
        let cmd_arr = cmd_str.split(" ").collect::<Vec<&str>>();
        if cmd_arr.len() < 2 {
            print!("命令错误！");
            return Err(ClientError::CliError("命令错误！".to_string()));
        }
        let code = cmd_arr[0];
        let cmd = match code {
            "get" => {
                let key = cmd_arr[1];
                let cmd = Command::Get { key };
                cmd
            }
            "set" if cmd_arr.len() == 3 => {
                let key = cmd_arr[1];
                let value = cmd_arr[2];
                let cmd = Command::Set {
                    key,
                    value: value.as_bytes(),
                };
                cmd
            }
            "delete" => {
                let key = cmd_arr[1];
                let cmd = Command::Delete { key };
                cmd
            }
            _ => {
                return Err(ClientError::CliError("命令错误！".to_string()));
            }
        };

        let cmd_code = cmd.encode();
        Ok(cmd_code)
    }

    /**
     * 解析用户输入
     */
    async fn user_input(&self, input: &str, stream: &mut TcpStream) -> ClientResult<bool> {
        let input = input.trim();
        if input == "exit" || input == "quit" {
            return Ok(false);
        }

        let cmd_code = self.input_decode(input)?;
        stream.write_all(&cmd_code).await?;
        stream.flush().await?;

        let mut buf = [0; 1024];
        let _n = stream.read(&mut buf).await?;

        if let Some(res) = Response::decode(&buf) {
            match res {
                Response::Ok => print!("OK"),
                Response::KeyNotExist => print!("KEY_NOT_EXIST"),
                Response::Deleted => print!("DELETED"),
                Response::Value(items) => {
                    let items_str = str::from_utf8(&items)?;
                    print!("{}", items_str.trim());
                }
                Response::Error(e) => print!("{}", e),
                _ => {}
            }
        }
        Ok(true)
    }
}
