use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use crate::data::Store;

pub async fn handler(command: Vec<String>, memory: Arc<Store>) -> String {
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

            memory.set(command[1].clone(), command[2].clone(), duration);
            "+OK\r\n".to_string()
        }
        "GET" => {
            if command.len() < 2 {
                return "-ERR wrong number of arguments for 'get' command\r\n".to_string();
            }
            match memory.get(&command[1]) {
                Some(value) => format!("+{}\r\n", value),
                None => "$-1\r\n".to_string(),
            }
        }
        "RPUSH" => {
            if command.len() < 3 {
                return "-ERR wrong number of arguments for 'rpush' command\r\n".to_string();
            }

            let mut len = 0;
            for value in command.iter().skip(2) {
                len = memory.rpush(command[1].clone(), value.clone()).await;
            }

            format!(":{}\r\n", len)
        }
        "LPUSH" => {
            if command.len() < 3 {
                return "-ERR wrong number of arguments for 'lpush' command\r\n".to_string();
            }

            let mut len = 0;
            for value in command.iter().skip(2) {
                len = memory.lpush(command[1].clone(), value.clone()).await;
            }

            format!(":{}\r\n", len)
        }
        "LPOP" => {
            if command.len() < 2 {
                return "-ERR wrong number of arguments for 'lpop' command\r\n".to_string();
            }

            let num_to_pop = if command.len() >= 3 {
                match command[2].parse::<usize>() {
                    Ok(num) => num,
                    Err(_) => {
                        return "-ERR value is not an integer or out of range\r\n".to_string();
                    }
                }
            } else {
                1
            };

            if num_to_pop > 1 {
                let mut response = format!("*{}\r\n", num_to_pop);
                for _ in 0..num_to_pop {
                    match memory.lpop(&command[1]) {
                        Some(value) => response.push_str(&format!("+{}\r\n", value)),
                        None => break,
                    }
                }
                return response;
            }

            match memory.lpop(&command[1]) {
                Some(value) => format!("+{}\r\n", value),
                None => "$-1\r\n".to_string(),
            }
        }
        "BLPOP" => {
            if command.len() < 3 {
                return "-ERR wrong number of arguments for 'blpop' command\r\n".to_string();
            }

            let timeout_secs = match command[2].parse::<u64>() {
                Ok(num) => num,
                Err(_) => {
                    return "-ERR timeout is not a float or out of range\r\n".to_string();
                }
            };

            let timeout = if timeout_secs == 0 {
                None
            } else {
                Some(SystemTime::now() + Duration::from_secs(timeout_secs))
            };

            match memory.blpop(&command[1], timeout).await {
                Some((key, value)) => format!("*2\r\n+{}\r\n+{}\r\n", key, value),
                None => "*-1\r\n".to_string(),
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

            let values = memory.lrange(&command[1], start, stop);

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

            format!(":{}\r\n", memory.llen(&command[1]))
        }
        _ => "-ERR unknown command\r\n".to_string(),
    }
}
