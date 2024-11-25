use std::io::{prelude, Error, Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() -> Result<(), Error> {
    let listner: TcpListener = TcpListener::bind("127.0.0.1:7878").map_err(|e| {
        eprintln!("Failed to start the server: {:?}", e);
        return e;
    })?;

    println!("Server has started on port 127.0.0.1:7878");

    for stream in listner.incoming() {
        let _ = match stream {
            Ok(s) => handle_connection(s),
            Err(e) => {
                eprintln!("Error accepting connecton: {:?}", e);
                return Err(e);
            }
        };
    }

    return Ok(());
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Error> {
    match handel_read(&mut stream) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    match handel_response(stream) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }
    return Ok(());
}

fn handel_read(mut stream: &TcpStream) -> Result<(), Error> {
    println!("New connection: {:?}", stream.peer_addr()?);
    // Create a byffer that is 4KB capacity
    let mut buffer = [0u8; 4096];

    match stream.read(&mut buffer) {
        Ok(_) => {
            let req_str = String::from_utf8_lossy(&buffer[..]);
            println!("Request: {}", req_str);
        }
        Err(e) => {
            eprintln!("Unable to read incoming stream: {:?}", e);
            return Err(e);
        }
    };

    return Ok(());
}

fn handel_response(mut stream: TcpStream) -> Result<(), Error> {
    let response  = b"HTTP/1.1 200 OK\r\nContent-Type: text/html;charset=UTF-8\r\n\r\n<html><body>Hello World!</body></html>\r\n";

    match stream.write(response) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error sending response: {:?}", e);
            return Err(e);
        }
    }

    match stream.flush() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error flushing the stream: {:?}", e);
            return Err(e);
        }
    }

    return Ok(());
}
