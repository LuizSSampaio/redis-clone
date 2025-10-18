pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Vec<RespValue>),
    Null,
    NullArray,
}

impl From<RespValue> for String {
    fn from(value: RespValue) -> String {
        match value {
            RespValue::SimpleString(s) => format!("+{}\r\n", s),
            RespValue::Error(e) => format!("-ERR {}\r\n", e),
            RespValue::Integer(n) => format!(":{}\r\n", n),
            RespValue::BulkString(Some(s)) => format!("${}\r\n{}\r\n", s.len(), s),
            RespValue::BulkString(None) | RespValue::Null => "$-1\r\n".to_string(),
            RespValue::NullArray => "*-1\r\n".to_string(),
            RespValue::Array(items) => {
                let mut result = format!("*{}\r\n", items.len());
                for item in items {
                    result.push_str(&String::from(item));
                }
                result
            }
        }
    }
}

pub fn parse(source: &[u8]) -> Vec<String> {
    let source = String::from_utf8_lossy(source).to_string();
    let mut res = Vec::new();

    source.split("\r\n").for_each(|line| {
        if line == "*" || line == "-" || line == "+" {
            res.push(line.to_string());
            return;
        }
        if line.starts_with('*') || line.starts_with('$') || line.is_empty() {
            return;
        }
        if line.starts_with('+') || line.starts_with(':') {
            res.push(line[1..].to_string());
            return;
        }
        res.push(line.to_string());
    });

    res
}
