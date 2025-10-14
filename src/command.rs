use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use crate::data::Store;
use crate::resp_parser::RespValue;

pub async fn handler(command: Vec<String>, memory: Arc<Store>) -> RespValue {
    if command.is_empty() {
        return RespValue::Error("unknown command".to_string());
    }

    match command[0].to_uppercase().as_str() {
        "PING" => RespValue::SimpleString("PONG".to_string()),
        "ECHO" => {
            if command.len() < 2 {
                return RespValue::Error(
                    "wrong number of arguments for 'echo' command".to_string(),
                );
            }
            RespValue::SimpleString(command[1].clone())
        }
        "SET" => {
            if command.len() < 3 {
                return RespValue::Error("wrong number of arguments for 'set' command".to_string());
            }

            let duration = match command.get(3) {
                Some(flag) => {
                    if command.len() < 5 {
                        return RespValue::Error(
                            "wrong number of arguments for 'set' command".to_string(),
                        );
                    }
                    let time = match command[4].parse::<u64>() {
                        Ok(time) => time,
                        Err(_) => {
                            return RespValue::Error(
                                "value is not an integer or out of range".to_string(),
                            );
                        }
                    };

                    match flag.to_uppercase().as_str() {
                        "EX" => Some(SystemTime::now() + Duration::from_secs(time)),
                        "PX" => Some(SystemTime::now() + Duration::from_millis(time)),
                        _ => {
                            return RespValue::Error("syntax error".to_string());
                        }
                    }
                }
                None => None,
            };

            memory.set(command[1].clone(), command[2].clone(), duration);
            RespValue::SimpleString("OK".to_string())
        }
        "GET" => {
            if command.len() < 2 {
                return RespValue::Error("wrong number of arguments for 'get' command".to_string());
            }
            match memory.get(&command[1]) {
                Some(value) => RespValue::BulkString(Some(value)),
                None => RespValue::Null,
            }
        }
        "RPUSH" => {
            if command.len() < 3 {
                return RespValue::Error(
                    "wrong number of arguments for 'rpush' command".to_string(),
                );
            }

            let mut len = 0;
            for value in command.iter().skip(2) {
                len = memory.rpush(command[1].clone(), value.clone()).await;
            }

            RespValue::Integer(len as i64)
        }
        "LPUSH" => {
            if command.len() < 3 {
                return RespValue::Error(
                    "wrong number of arguments for 'lpush' command".to_string(),
                );
            }

            let mut len = 0;
            for value in command.iter().skip(2) {
                len = memory.lpush(command[1].clone(), value.clone()).await;
            }

            RespValue::Integer(len as i64)
        }
        "LPOP" => {
            if command.len() < 2 {
                return RespValue::Error(
                    "wrong number of arguments for 'lpop' command".to_string(),
                );
            }

            let num_to_pop = if command.len() >= 3 {
                match command[2].parse::<usize>() {
                    Ok(num) => num,
                    Err(_) => {
                        return RespValue::Error(
                            "value is not an integer or out of range".to_string(),
                        );
                    }
                }
            } else {
                1
            };

            if num_to_pop > 1 {
                let mut values = Vec::new();
                for _ in 0..num_to_pop {
                    match memory.lpop(&command[1]) {
                        Some(value) => values.push(RespValue::BulkString(Some(value))),
                        None => break,
                    }
                }
                return RespValue::Array(values);
            }

            match memory.lpop(&command[1]) {
                Some(value) => RespValue::BulkString(Some(value)),
                None => RespValue::Null,
            }
        }
        "BLPOP" => {
            if command.len() < 3 {
                return RespValue::Error(
                    "wrong number of arguments for 'blpop' command".to_string(),
                );
            }

            let timeout_secs = match command[2].parse::<f64>() {
                Ok(num) => num,
                Err(_) => {
                    return RespValue::Error("timeout is not a float or out of range".to_string());
                }
            };

            let timeout = if timeout_secs == 0.0 {
                None
            } else {
                Some(SystemTime::now() + Duration::from_secs_f64(timeout_secs))
            };

            match memory.blpop(&command[1], timeout).await {
                Some((key, value)) => RespValue::Array(vec![
                    RespValue::BulkString(Some(key)),
                    RespValue::BulkString(Some(value)),
                ]),
                None => RespValue::NullArray,
            }
        }
        "LRANGE" => {
            if command.len() < 4 {
                return RespValue::Error(
                    "wrong number of arguments for 'lrange' command".to_string(),
                );
            }

            let start = match command[2].parse::<isize>() {
                Ok(num) => num,
                Err(_) => {
                    return RespValue::Error("value is not an integer or out of range".to_string());
                }
            };

            let stop = match command[3].parse::<isize>() {
                Ok(num) => num,
                Err(_) => {
                    return RespValue::Error("value is not an integer or out of range".to_string());
                }
            };

            let values = memory.lrange(&command[1], start, stop);

            RespValue::Array(
                values
                    .into_iter()
                    .map(|v| RespValue::BulkString(Some(v)))
                    .collect(),
            )
        }
        "LLEN" => {
            if command.len() < 2 {
                return RespValue::Error(
                    "wrong number of arguments for 'llen' command".to_string(),
                );
            }

            RespValue::Integer(memory.llen(&command[1]) as i64)
        }
        _ => RespValue::Error("unknown command".to_string()),
    }
}
