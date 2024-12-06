pub mod connections {
    #![allow(dead_code, unused_variables)]

    use std::net::SocketAddr;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::broadcast::Sender;
    use tokio::sync::{broadcast, Mutex, Semaphore};
    use tokio::{fs, time};

    use crate::request_validation::handle_request;
    use crate::shutdown::Message;
    use crate::ErrorType;

    const MAX_CONNECTIONS: usize = 5;

    #[derive(Debug)]
    pub struct Listener {
        pub listener: TcpListener,
        pub connection_limit: Arc<Semaphore>,
        pub shutdown_tx: Arc<Mutex<Sender<Message>>>,
    }

    #[derive(Debug)]
    pub struct ConnectionHandler {
        pub stream: TcpStream,
        pub addr: SocketAddr,
        pub shutdown_rx: broadcast::Receiver<Message>,
    }

    pub async fn handle_connection(stream: &mut TcpStream) -> Result<(), ErrorType> {
        loop {
            let mut buffer = [0; 4096];

            let bytes_read: usize = match stream.read(&mut buffer).await {
                Ok(n) => {
                    if n == 0 {
                        return Ok(());
                    } else {
                        n
                    }
                }
                Err(e) => {
                    let error: ErrorType =
                        ErrorType::SocketError(String::from("Failed to read from socket"));
                    return Err(error);
                }
            };

            handle_request(&buffer[..bytes_read])?;

            if buffer.starts_with(get_route("test")) {
                format_response(
                    "200 OK",
                    fs::read_to_string("html/home.html").await.unwrap(),
                    stream,
                )
                .await;
            } else if buffer.starts_with(get_route("hayley")) {
                thread::sleep(Duration::from_secs(5));
                format_response(
                    "200 OK",
                    fs::read_to_string("html/index.html").await.unwrap(),
                    stream,
                )
                .await;
            } else {
                format_response(
                    "200 OK",
                    fs::read_to_string("html/index.html").await.unwrap(),
                    stream,
                )
                .await;
            }
        }
    }

    pub async fn format_response(status_code: &str, contents: String, stream: &mut TcpStream) {
        let length: usize = contents.len();
        let response =
            format!("HTTP/1.1 {status_code}\r\nContent-Length: {length}\r\n\r\n{contents}");
        stream.write_all(response.as_bytes()).await.unwrap();
    }

    pub fn get_route(route: &str) -> &'static [u8] {
        return match route {
            "Home" => b"GET / HTTP/1.1",
            "hayley" => b"GET /hayley HTTP/1.1",
            "test" => b"GET /home HTTP/1.1",
            _ => b"GET / HTTP/1.1",
        };
    }

    pub fn validate_request(req: &[u8]) -> Result<(), ErrorType> {
        return Ok(());
    }

    impl Listener {
        pub async fn accept(&mut self) -> Result<(TcpStream, SocketAddr), ErrorType> {
            let mut backoff: usize = 200;

            loop {
                // If socket it accepted then return the associated handler
                match self.listener.accept().await {
                    Ok((stream, addr)) => {
                        println!("New connection from {}", addr);
                        return Ok((stream, addr));
                    }
                    Err(_) => {
                        // Attempt has failed too many times
                        if backoff > 6000 {
                            return Err(ErrorType::SocketError(String::from(
                                "Error establishing connection",
                            )));
                        }
                    }
                }

                // Exponential backoff to reduce contention
                println!("Backingoff...");
                time::sleep(Duration::from_millis(backoff as u64)).await;
                backoff *= 2;
            }
        }
    }
}
