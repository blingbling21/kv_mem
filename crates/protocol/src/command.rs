use crate::error::{ProtocolError::ParseError, ProtocolResult};

#[derive(Debug)]
pub enum Command<'a> {
    Get { key: &'a str },
    Set { key: &'a str, value: &'a [u8] },
    Delete { key: &'a str },
}

impl<'a> Command<'a> {
    /// 将 Command 枚举转换成字节流
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Command::Get { key } => {
                let mut buf = vec![1];
                buf.extend_from_slice(&(key.len() as u16).to_be_bytes());
                buf.extend_from_slice(key.as_bytes());
                buf
            }
            Command::Set { key, value } => {
                let mut buf = vec![2];
                buf.extend_from_slice(&(key.len() as u16).to_be_bytes());
                buf.extend_from_slice(key.as_bytes());
                buf.extend_from_slice(&(value.len() as u16).to_be_bytes());
                buf.extend_from_slice(value);
                buf
            }
            Command::Delete { key } => {
                let mut buf = vec![3];
                buf.extend_from_slice(&(key.len() as u16).to_be_bytes());
                buf.extend_from_slice(key.as_bytes());
                buf
            }
        }
    }

    /// 解析字节流为 Command 枚举，返回Command类型和指令长度
    pub fn decode(raw_cmd: &'a [u8]) -> ProtocolResult<(Command<'a>, usize)> {
        if raw_cmd.len() < 3 {
            return Err(ParseError("解析失败，字节长度不足。".to_string()));
        }
        let command_code = raw_cmd[0];
        let array = [raw_cmd[1], raw_cmd[2]];
        let key_len = u16::from_be_bytes(array) as usize;

        if raw_cmd.len() < 3 + key_len {
            return Err(ParseError("解析失败，字节长度不足以包含键。".to_string()));
        }

        let key = std::str::from_utf8(&raw_cmd[3..3 + key_len])?;

        match command_code {
            1 => Ok((Command::Get { key }, 3 + key_len)),
            2 => {
                if raw_cmd.len() < 3 + key_len + 2 {
                    return Err(ParseError(
                        "解析失败，字节长度不足以包含值长度。".to_string(),
                    ));
                }
                let value_len =
                    u16::from_be_bytes([raw_cmd[3 + key_len], raw_cmd[4 + key_len]]) as usize;
                if raw_cmd.len() < 3 + key_len + 2 + value_len {
                    return Err(ParseError("解析失败，字节长度不足以包含值。".to_string()));
                }
                let value = &raw_cmd[5 + key_len..5 + key_len + value_len];
                return Ok((Command::Set { key, value }, 5 + key_len + value_len));
            }
            3 => Ok((Command::Delete { key }, 3 + key_len)),
            _ => Err(ParseError("解析失败，未知的Command类型。".to_string())),
        }
    }
}
