use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use tokio::sync::Mutex;

use crate::data::Store;

pub async fn handler(command: Vec<String>, memory: Arc<Mutex<Store>>) -> String {
    if command.is_empty() {
        return "-ERR unknown command\r\n".to_string();
    }

    match command[0].to_uppercase().as_str() {
        "PING" => "+PONG\r\n".to_string(),
        "ECHO" => {
            if command.len() < 2 {
                return "-ERR wrong number of arguments for 'echo' command\r\n".to_string();
            }
            format!("+{}\r\n", command[1])
        }
        "SET" => {
            if command.len() < 3 {
                return "-ERR wrong number of arguments for 'set' command\r\n".to_string();
            }

            let duration = match command.get(3) {
                Some(flag) => {
                    if command.len() < 5 {
                        return "-ERR wrong number of arguments for 'set' command\r\n".to_string();
                    }
                    let time = match command[4].parse::<u64>() {
                        Ok(time) => time,
                        Err(_) => {
                            return "-ERR value is not an integer or out of range\r\n".to_string();
                        }
                    };

                    match flag.to_uppercase().as_str() {
                        "EX" => Some(SystemTime::now() + Duration::from_secs(time)),
                        "PX" => Some(SystemTime::now() + Duration::from_millis(time)),
                        _ => {
                            return "-ERR syntax error\r\n".to_string();
                        }
                    }
                }
                None => None,
            };

            let mem = memory.lock().await;
            mem.set(command[1].clone(), command[2].clone(), duration);
            "+OK\r\n".to_string()
        }
        "GET" => {
            if command.len() < 2 {
                return "-ERR wrong number of arguments for 'get' command\r\n".to_string();
            }
            let mem = memory.lock().await;
            match mem.get(&command[1]) {
                Some(value) => format!("+{}\r\n", value),
                None => "$-1\r\n".to_string(),
            }
        }
        "RPUSH" => {
            if command.len() < 3 {
                return "-ERR wrong number of arguments for 'rpush' command\r\n".to_string();
            }

            let mem = memory.lock().await;
            let mut len = 0;
            for value in command.iter().skip(2) {
                len = mem.rpush(command[1].clone(), value.clone());
            }

            format!(":{}\r\n", len)
        }
        "LPUSH" => {
            if command.len() < 3 {
                return "-ERR wrong number of arguments for 'lpush' command\r\n".to_string();
            }

            let mem = memory.lock().await;
            let mut len = 0;
            for value in command.iter().skip(2) {
                len = mem.lpush(command[1].clone(), value.clone());
            }

            format!(":{}\r\n", len)
        }
        "LPOP" => {
            if command.len() < 2 {
                return "-ERR wrong number of arguments for 'lpop' command\r\n".to_string();
            }

            let mem = memory.lock().await;
            match mem.lpop(&command[1]) {
                Some(value) => format!("+{}\r\n", value),
                None => "$-1\r\n".to_string(),
            }
        }
        "LRANGE" => {
            if command.len() < 4 {
                return "-ERR wrong number of arguments for 'lrange' command\r\n".to_string();
            }

            let start = match command[2].parse::<isize>() {
                Ok(num) => num,
                Err(_) => {
                    return "-ERR value is not an integer or out of range\r\n".to_string();
                }
            };

            let stop = match command[3].parse::<isize>() {
                Ok(num) => num,
                Err(_) => {
                    return "-ERR value is not an integer or out of range\r\n".to_string();
                }
            };

            let mem = memory.lock().await;
            let values = mem.lrange(&command[1], start, stop);

            let mut response = format!("*{}\r\n", values.len());
            for value in values {
                response.push_str(&format!("+{}\r\n", value));
            }
            response
        }
        "LLEN" => {
            if command.len() < 2 {
                return "-ERR wrong number of arguments for 'llen' command\r\n".to_string();
            }

            let mem = memory.lock().await;
            format!(":{}\r\n", mem.llen(&command[1]))
        }
        _ => "-ERR unknown command\r\n".to_string(),
    }
}
