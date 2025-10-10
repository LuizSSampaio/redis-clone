use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod resp_parser;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(mut socket: TcpStream) {
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = socket.read(&mut buffer).await.unwrap();
        if bytes_read == 0 {
            break;
        }

        let content = resp_parser::parse(&buffer[..bytes_read]);
        println!("Received: {:?}", content);
        socket.write_all(b"+PONG\r\n").await.unwrap();
    }
}
