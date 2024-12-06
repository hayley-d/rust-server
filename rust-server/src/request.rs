use crate::ErrorType;
use chrono::{DateTime, Utc};
use core::str;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fmt::Display;
use std::io::Write;

pub enum Protocol {
    Http,
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Http => write!(f, "HTTP/1.1"),
        }
    }
}

pub enum ContentType {
    Text,
    Html,
    Json,
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Text => write!(f, "text/plain"),
            ContentType::Html => write!(f, "text/html"),
            ContentType::Json => write!(f, "application/json"),
        }
    }
}

pub struct Response {
    pub protocol: Protocol,
    pub code: HttpCode,
    pub content_type: ContentType,
    pub body: Vec<u8>,
    pub compression: bool,
}

impl Response {
    pub fn to_bytes(&self) -> Vec<u8> {
        // Response line: HTTP/1.1 <status code>
        let response_line: String = format!("{} {}\r\n", self.protocol, self.code);

        // Date Header
        let now: DateTime<Utc> = Utc::now();
        let date = now.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let mut headers: Vec<String> = vec![
            format!("Server: Ferriscuit"),
            format!("Date: {}", date),
            format!("Cache-Control: no-cache"),
            format!("Content-Type: {}", self.content_type),
        ];

        let body: Vec<u8>;

        if !self.compression {
            body = self.body.clone();
        } else {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder
                .write_all(&self.body)
                .expect("Failed to write body to gzip encoder");
            body = encoder.finish().expect("Failed to finish gzip compression");
            headers.push(format!("Content-Encoding: gzip"));
        }
        headers.push(format!("Content-Length: {}", body.len()));

        let mut response = Vec::new();
        response.extend_from_slice(response_line.as_bytes());
        response.extend_from_slice(headers.join("\r\n").as_bytes());
        response.extend_from_slice(b"\r\n\r\n");
        response.extend_from_slice(&body);

        return response;
    }
}

pub struct Request {
    pub headers: Vec<String>,
    pub body: String,
    pub method: HttpMethod,
    pub uri: String,
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
                let key_words: [&str; 5] = ["Host", "User-Agent", "Accept", "Encoding", "Brew"];
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

    pub fn is_compression_supported(&self) -> bool {
        for header in &self.headers {
            let header = header.to_lowercase();

            if header.contains("accept-encoding") {
                if header.to_lowercase().contains(',') {
                    // multiple compression types
                    let mut encodings: Vec<&str> =
                        header.split(',').map(|m| m.trim()).collect::<Vec<&str>>();
                    encodings[0] = &encodings[0].split_whitespace().collect::<Vec<&str>>()[1];

                    for encoding in encodings {
                        if encoding == "gzip" {
                            return true;
                        }
                    }
                } else {
                    if header
                        .to_lowercase()
                        .split_whitespace()
                        .collect::<Vec<&str>>()[1]
                        == "gzip"
                    {
                        return true;
                    }
                }
            }
        }
        return false;
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
