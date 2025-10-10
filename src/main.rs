use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        process(socket).await;
    }
}

async fn process(socket: TcpStream) {
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = socket.try_read(&mut buffer).unwrap();
        if bytes_read == 0 {
            break;
        }

        socket.try_write(b"+PONG\r\n").unwrap();
    }
}
