use std::thread;
use std::time::Duration;

use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;

use crate::{ContentType, HttpCode, Protocol, Request, Response};

async fn read_file_to_bytes(path: &str) -> Vec<u8> {
    let metadata = fs::metadata(path).await.unwrap();
    let mut file = File::open(path).await.unwrap();
    let mut buffer: Vec<u8> = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut buffer).await.unwrap();
    return buffer;
}

pub async fn handle_get(request: Request) -> Response {
    if request.uri == "/" {
        return Response {
            protocol: Protocol::Http,
            code: HttpCode::Ok,
            content_type: ContentType::Html,
            body: read_file_to_bytes("html/index.html").await,
        };
    } else if request.uri == "/hayley" {
        thread::sleep(Duration::from_secs(5));
        return Response {
            protocol: Protocol::Http,
            code: HttpCode::Ok,
            content_type: ContentType::Html,
            body: read_file_to_bytes("html/index.html").await,
        };
    } else if request.uri == "/home" {
        return Response {
            protocol: Protocol::Http,
            code: HttpCode::Ok,
            content_type: ContentType::Html,
            body: read_file_to_bytes("html/home.html").await,
        };
    } else {
        return Response {
            protocol: Protocol::Http,
            code: HttpCode::Ok,
            content_type: ContentType::Html,
            body: read_file_to_bytes("html/index.html").await,
        };
    }
}

/*fn handle_post(request: Request) {
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
}*/
