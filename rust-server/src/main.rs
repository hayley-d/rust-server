use std::io::Error;
use std::net::{TcpListener, TcpStream};
fn main() {
    let listner: TcpListener = match TcpListener::bind("127.0.0.1:7878") {
        Ok(l) => l,
        Err(e) => panic!("Error occured when starting server: {:?}", e),
    };

    for stream in listner.incoming() {
        let stream: TcpStream = match stream {
            Ok(s) => s,
            Err(e) => panic!("Error occured when checking port stream {:?}", e),
        };
    }
}
