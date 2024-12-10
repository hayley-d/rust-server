use crate::{read_file_to_bytes, ErrorType};
use chrono::{DateTime, Utc};
use core::str;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fmt::Display;
use std::io::Write;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Header {
    pub title: String,
    pub value: String,
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} : {}", self.title, self.value)
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Response {
    pub protocol: Protocol,
    pub code: HttpCode,
    pub content_type: ContentType,
    pub body: Vec<u8>,
    pub compression: bool,
    pub headers: Vec<Header>,
}

#[allow(async_fn_in_trait)]
pub trait MyDefault {
    async fn default() -> Self;
}

impl MyDefault for Response {
    async fn default() -> Self {
        let mut response = Response::new(Protocol::Http, HttpCode::Ok, ContentType::Html, true);

        response.add_body(read_file_to_bytes("static/index.html").await);

        return response;
    }
}

impl Response {
    pub fn add_header(&mut self, title: String, value: String) {
        self.headers.push(Header { title, value });
    }

    pub fn to_bytes(&mut self) -> Vec<u8> {
        // Response line: HTTP/1.1 <status code>
        let response_line: String = format!("{} {}\r\n", self.protocol, self.code);

        let body: Vec<u8>;

        if !self.compression {
            body = self.body.clone();
        } else {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder
                .write_all(&self.body)
                .expect("Failed to write body to gzip encoder");
            body = encoder.finish().expect("Failed to finish gzip compression");
            //self.add_header(String::from("Content-Encoding"), String::from("gzip"));
        }

        self.add_header(String::from("Content-Length"), body.len().to_string());

        let mut headers: Vec<String> = Vec::new();

        for header in &self.headers {
            headers.push(header.to_string());
        }

        println!("{:?}", headers);

        let mut response = Vec::new();
        response.extend_from_slice(response_line.as_bytes());
        response.extend_from_slice(headers.join("\r\n").as_bytes());
        response.extend_from_slice(b"\r\n\r\n");
        response.extend_from_slice(&body);

        return response;
    }

    pub fn add_body(&mut self, body: Vec<u8>) {
        self.body = body;
    }

    pub fn new(
        protocol: Protocol,
        code: HttpCode,
        content_type: ContentType,
        compression: bool,
    ) -> Self {
        let body = Vec::with_capacity(0);

        // Date Header
        let now: DateTime<Utc> = Utc::now();
        let date = now.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let mut headers: Vec<Header> = vec![
            Header {
                title: String::from("Server"),
                value: String::from("Ferriscuit"),
            },
            Header {
                title: String::from("Date"),
                value: date,
            },
            Header {
                title: String::from("Cache-Control"),
                value: String::from("no-cache"),
            },
            Header {
                title: String::from("Content-Type"),
                value: content_type.to_string(),
            },
        ];

        if compression {
            headers.push(Header {
                title: String::from("Content-Encoding"),
                value: String::from("gzip"),
            });
        }

        return Response {
            protocol,
            code,
            content_type,
            body,
            compression,
            headers,
        };
    }

    pub fn code(mut self, code: HttpCode) -> Self {
        self.code = code;
        return self;
    }

    pub fn content_type(mut self, content_type: ContentType) -> Self {
        self.content_type = content_type;
        return self;
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        return self;
    }

    pub fn compression(mut self, compression: bool) -> Self {
        self.compression = compression;
        // add header
        if compression {
            for header in &self.headers {
                if header.title == "Content-Encoding" {
                    return self;
                }
            }
            self.add_header(String::from("Content-Encoding"), String::from("gzip"));
        } else {
            let mut index: isize = -1;
            for (i, _) in self.headers.iter().enumerate() {
                if &self.headers[i].title == "Content-Encoding" {
                    index = i as isize;
                }
            }

            if index > 0 {
                self.headers.remove(index as usize);
            }
        }
        return self;
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

        println!("{}\r\n", request);

        // split the request by line
        let request: Vec<&str> = request.lines().collect();

        if request.len() < 3 {
            return Err(ErrorType::ConnectionError(String::from("Invalid request")));
        }

        // get the http method from the first line
        let method: HttpMethod =
            HttpMethod::new(request[0].split_whitespace().collect::<Vec<&str>>()[0]);

        // get the uri from the first line
        let uri: String = request[0].split_whitespace().collect::<Vec<&str>>()[1].to_string();

        // headers are the rest of the
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
                headers.push(line.to_string());
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

            if header.contains("firefox") {
                return false;
            }

            if header.contains("accept-encoding") {
                if header.contains(',') {
                    // multiple compression types
                    let mut encodings: Vec<&str> =
                        header.split(", ").map(|m| m.trim()).collect::<Vec<&str>>();
                    encodings[0] = &encodings[0].split_whitespace().collect::<Vec<&str>>()[1];

                    for encoding in encodings {
                        if encoding == "gzip" || encoding.contains("gzip") {
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
