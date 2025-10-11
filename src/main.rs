use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::memory::Memory;

mod command;
mod data;
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
        let response = command::handler(content, memory.clone()).await;
        socket.write_all(response.as_bytes()).await.unwrap();
    }
}
