use tokio::fs;
use tokio::net::TcpStream;

use crate::ErrorType;
use std::fmt::Display;

pub struct Request {
    headers: Vec<String>,
    body: String,
    method: HttpMethod,
    uri: String,
}

impl Request {
    pub fn new(buffer: &[u8]) -> Result<Request, ErrorType> {
        // unwrap is safe as request has been parsed for any issues before this is called
        let request = String::from_utf8(buffer.to_vec()).unwrap();

        let request: Vec<&str> = request.lines().collect();

        if request.len() < 3 {
            return Err(ErrorType::ConnectionError(String::from("Invalid request")));
        }

        let method: HttpMethod =
            HttpMethod::new(request[0].split_whitespace().collect::<Vec<&str>>()[0]);

        let uri: String = request[0].split_whitespace().collect::<Vec<&str>>()[1].to_string();

        let mut headers: Vec<String> = Vec::with_capacity(request.len() - 1);
        let mut body: String = String::new();
        let mut flag = false;
        for line in &request[1..] {
            if line.is_empty() {
                flag = true;
                continue;
            }
            if flag {
                body.push_str(line);
            } else {
                let key_words: [&str; 4] = ["Host", "User-Agent", "Accept", "Encoding"];
                for word in key_words {
                    if line.contains(word) {
                        headers.push(line.to_string());
                    }
                }
            }
        }

        return Ok(Request {
            headers,
            body,
            method,
            uri,
        });
    }
}

#[derive(Debug)]
pub enum HttpCode {
    Ok,
    Created,
    BadRequest,
    Unauthorized,
    NotFound,
    MethodNotAllowed,
    RequestTimeout,
    Teapot,
    InternalServerError,
}

impl Display for HttpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpCode::Ok => write!(f, "200 OK"),
            HttpCode::Created => write!(f, "201 Created"),
            HttpCode::BadRequest => write!(f, "400 Bad Request"),
            HttpCode::Unauthorized => write!(f, "401 Unauthorized"),
            HttpCode::NotFound => write!(f, "404 Not Found"),
            HttpCode::MethodNotAllowed => write!(f, "405 Method Not Allowed"),
            HttpCode::RequestTimeout => write!(f, "408 Request Timeout"),
            HttpCode::Teapot => write!(f, "418 I'm a teapot"),
            HttpCode::InternalServerError => write!(f, "500 Internal Server Error"),
        }
    }
}

impl PartialEq for HttpCode {
    fn eq(&self, other: &Self) -> bool {
        match self {
            HttpCode::Ok => match other {
                HttpCode::Ok => true,
                _ => false,
            },
            HttpCode::Created => match other {
                HttpCode::Created => true,
                _ => false,
            },
            HttpCode::BadRequest => match other {
                HttpCode::BadRequest => true,
                _ => false,
            },
            HttpCode::Unauthorized => match other {
                HttpCode::Unauthorized => true,
                _ => false,
            },
            HttpCode::NotFound => match other {
                HttpCode::NotFound => true,
                _ => false,
            },
            HttpCode::MethodNotAllowed => match other {
                HttpCode::MethodNotAllowed => true,
                _ => false,
            },
            HttpCode::RequestTimeout => match other {
                HttpCode::RequestTimeout => true,
                _ => false,
            },
            HttpCode::Teapot => match other {
                HttpCode::Teapot => true,
                _ => false,
            },
            HttpCode::InternalServerError => match other {
                HttpCode::InternalServerError => true,
                _ => false,
            },
        }
    }
}

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

impl HttpMethod {
    pub fn new(method: &str) -> HttpMethod {
        if method.to_uppercase().contains("GET") {
            HttpMethod::GET
        } else if method.to_uppercase().contains("POST") {
            HttpMethod::POST
        } else if method.to_uppercase().contains("PUT") {
            HttpMethod::PUT
        } else if method.to_uppercase().contains("PATCH") {
            HttpMethod::PATCH
        } else {
            HttpMethod::DELETE
        }
    }
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::PATCH => write!(f, "PATCH"),
            HttpMethod::DELETE => write!(f, "DELETE"),
        }
    }
}

impl PartialEq for HttpMethod {
    fn eq(&self, other: &Self) -> bool {
        match self {
            HttpMethod::GET => match other {
                HttpMethod::GET => true,
                _ => false,
            },
            HttpMethod::POST => match other {
                HttpMethod::POST => true,
                _ => false,
            },
            HttpMethod::PUT => match other {
                HttpMethod::PUT => true,
                _ => false,
            },
            HttpMethod::PATCH => match other {
                HttpMethod::PATCH => true,
                _ => false,
            },
            HttpMethod::DELETE => match other {
                HttpMethod::DELETE => true,
                _ => false,
            },
        }
    }
}
pub async fn format_response(status_code: &str, contents: String, stream: &mut TcpStream) {
    let length: usize = contents.len();
    let response = format!("HTTP/1.1 {status_code}\r\nContent-Length: {length}\r\n\r\n{contents}");
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

async fn handle_get(request: Request, stream: &mut TcpStream) {
    if request.uri == "/" {
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

    todo!()
}

fn handle_post(request: Request) {
    todo!()
}

fn handle_put(request: Request) {
    todo!()
}

fn handle_patch(request: Request) {
    todo!()
}

fn handle_delete(request: Request) {
    todo!()
}
