use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::memory::Memory;

mod memory;
mod resp_parser;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let memory = Arc::new(Mutex::new(Memory::default()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let memory_clone = memory.clone();
        tokio::spawn(async move {
            process(socket, memory_clone).await;
        });
    }
}

async fn process(mut socket: TcpStream, memory: Arc<Mutex<Memory>>) {
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = socket.read(&mut buffer).await.unwrap();
        if bytes_read == 0 {
            break;
        }

        let content = resp_parser::parse(&buffer[..bytes_read]);
        let response = command_handler(content, memory.clone()).await;
        socket.write_all(response.as_bytes()).await.unwrap();
    }
}

async fn command_handler(command: Vec<String>, memory: Arc<Mutex<Memory>>) -> String {
    if command.is_empty() {
        return "-ERR unknown command\r\n".to_string();
    }

    match command[0].to_uppercase().as_str() {
        "PING" => "+PONG\r\n".to_string(),
        "ECHO" => {
            if command.len() < 2 {
                "-ERR wrong number of arguments for 'echo' command\r\n".to_string()
            } else {
                format!("+{}\r\n", command[1])
            }
        }
        "SET" => {
            if command.len() < 3 {
                "-ERR wrong number of arguments for 'set' command\r\n".to_string()
            } else {
                memory
                    .lock()
                    .await
                    .set(command[1].clone(), command[2].clone());
                "+OK\r\n".to_string()
            }
        }
        "GET" => {
            if command.len() < 2 {
                "-ERR wrong number of arguments for 'get' command\r\n".to_string()
            } else {
                let mem = memory.lock().await;
                match mem.get(&command[1]) {
                    Some(value) => format!("+{}\r\n", value),
                    None => "$-1\r\n".to_string(),
                }
            }
        }
        _ => "-ERR unknown command\r\n".to_string(),
    }
}
