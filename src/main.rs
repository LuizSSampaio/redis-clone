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
        let response = command_handler(content);
        socket.write_all(response.as_bytes()).await.unwrap();
    }
}

fn command_handler(command: Vec<String>) -> String {
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
        _ => "-ERR unknown command\r\n".to_string(),
    }
}
