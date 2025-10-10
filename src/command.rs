use std::sync::Arc;

use tokio::sync::Mutex;

use crate::memory::Memory;

pub async fn handler(command: Vec<String>, memory: Arc<Mutex<Memory>>) -> String {
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
            let mut mem = memory.lock().await;
            mem.set(command[1].clone(), command[2].clone());
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
        _ => "-ERR unknown command\r\n".to_string(),
    }
}
