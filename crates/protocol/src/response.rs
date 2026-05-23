const ERROR_PREFIX: &str = "ERROR: ";

/// 定义命令执行结果的枚举类型
pub enum Response {
    Ok,
    KeyNotExist,
    Deleted,
    Value(Vec<u8>),
    Error(String),
}

impl Response {
    /// 将 Response 转换成字节流
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Response::Ok => b"Ok\n".to_vec(),
            Response::KeyNotExist => b"KEY_NOT_EXIST\n".to_vec(),
            Response::Deleted => b"DELETED\n".to_vec(),
            Response::Value(val) => {
                let mut value = val.clone();
                value.push(b'\n');
                value
            }
            Response::Error(msg) => format!("{}{}\n", ERROR_PREFIX, msg).into_bytes(),
        }
    }

    /// 从字节流解析出 Response 枚举
    pub fn decode(input: &[u8]) -> Option<Self> {
        if input.is_empty() {
            return None;
        }

        let input_str = std::str::from_utf8(input).ok()?.trim_end();

        match input_str {
            "Ok" => Some(Self::Ok),
            "KEY_NOT_EXIST" => Some(Self::KeyNotExist),
            "DELETED" => Some(Self::Deleted),
            s if s.starts_with(ERROR_PREFIX) => {
                Some(Self::Error(s[ERROR_PREFIX.len()..].to_string()))
            }
            _ => Some(Self::Value(input_str.as_bytes().to_vec())), // 默认当作值处理
        }
    }
}
