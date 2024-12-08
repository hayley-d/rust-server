use crate::{ContentType, ErrorType, HttpCode, HttpMethod, MyDefault, Request, Response};
use argon2::Argon2;
use rand::Rng;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn read_file_to_bytes(path: &str) -> Vec<u8> {
    let metadata = fs::metadata(path).await.unwrap();
    let mut file = File::open(path).await.unwrap();
    let mut buffer: Vec<u8> = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut buffer).await.unwrap();
    return buffer;
}

pub async fn handle_response(request: Request) -> Response {
    match request.method {
        HttpMethod::GET => handle_get(request).await,
        HttpMethod::POST => handle_post(request).await,
        HttpMethod::PUT => handle_put(request).await,
        HttpMethod::PATCH => handle_patch(request).await,
        HttpMethod::DELETE => handle_delete(request).await,
    }
}

async fn handle_get(request: Request) -> Response {
    if request.headers.contains(&String::from("Brew")) || request.uri == "/coffee" {
        let response = Response::default()
            .await
            .code(HttpCode::Teapot)
            .content_type(ContentType::Text)
            .compression(request.is_compression_supported())
            .body(
                r#"
      I'm a Teapot, I can't brew coffee
         _______
        /       \
       |  O   O |
       |    ^    |
        \_______/
"#
                .as_bytes()
                .to_vec(),
            );

        return response;
    }

    let mut response = Response::default()
        .await
        .compression(request.is_compression_supported());

    if request.uri == "/" {
        // Add Response Body
        response.add_body(read_file_to_bytes("static/index.html").await);
    } else if request.uri == "/hayley" {
        thread::sleep(Duration::from_secs(5));

        response.add_body(read_file_to_bytes("static/index.html").await);
    } else if request.uri == "/home" {
        response.add_body(read_file_to_bytes("static/home.html").await);
    } else {
        response.add_body(read_file_to_bytes("static/index.html").await);
    }
    return response;
}

async fn handle_post(request: Request) -> Response {
    let mut response = Response::default()
        .await
        .compression(request.is_compression_supported())
        .body(read_file_to_bytes("static/index.html").await);

    if request.uri == "/signup" {
        // parse the JSON into a hashmap
        let user: HashMap<String, String> = match serde_json::from_str(&request.body) {
            Ok(u) => u,
            Err(_) => {
                return response.code(HttpCode::InternalServerError);
            }
        };
        let session_id: String = generate_session_id();

        // insert the new user into the file
        match insert_user(
            user["username"].clone(),
            user["password"].clone(),
            session_id.clone(),
        )
        .await
        {
            Ok(_) => (),
            Err(_) => {
                return response.code(HttpCode::InternalServerError);
            }
        }

        response.add_header(
            String::from("Set-Cookie"),
            format!("session={}; HttpOnly", session_id),
        );

        return response;
    }

    return response.code(HttpCode::InternalServerError);
}

async fn handle_put(request: Request) -> Response {
    let response = Response::default()
        .await
        .compression(request.is_compression_supported())
        .body(read_file_to_bytes("static/index.html").await)
        .code(HttpCode::MethodNotAllowed);

    return response;
}

async fn handle_patch(request: Request) -> Response {
    let response = Response::default()
        .await
        .compression(request.is_compression_supported())
        .body(read_file_to_bytes("static/index.html").await)
        .code(HttpCode::MethodNotAllowed);

    return response;
}

async fn handle_delete(request: Request) -> Response {
    let response = Response::default()
        .await
        .compression(request.is_compression_supported())
        .body(read_file_to_bytes("static/index.html").await)
        .code(HttpCode::MethodNotAllowed);

    return response;
}

async fn insert_user(username: String, password: String, session: String) -> Result<(), ErrorType> {
    let password = password.into_bytes();
    let mut hashed_password: Vec<u8> = Vec::new();
    match Argon2::default().hash_password_into(&password, b"", &mut hashed_password) {
        Ok(_) => (),
        Err(_) => {
            return Err(ErrorType::InternalServerError(String::from(
                "Problem occured when creating password",
            )));
        }
    }

    let mut file_input: Vec<u8> = username.into_bytes();
    file_input.push(b"|"[0]);
    file_input.append(&mut hashed_password);
    file_input.push(b"|"[0]);
    file_input.append(&mut session.into_bytes());

    let mut file = OpenOptions::new()
        .append(true)
        .open("/static/users.txt")
        .await
        .expect("cannot open file");

    match file.write(&file_input).await {
        Ok(_) => (),
        Err(_) => {
            return Err(ErrorType::InternalServerError(String::from(
                "Problem occured when writing user to db",
            )));
        }
    };

    Ok(())
}

fn generate_session_id() -> String {
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect()
}

async fn verify_cookie(cookie: &str) -> bool {
    if cookie.starts_with("session=") {
        return match fs::read_to_string("static/users.txt").await {
            Ok(f) => {
                let cookie_value: &str = cookie.split('=').collect::<Vec<&str>>()[1];
                println!("Cookie value: {}", cookie_value);
                f.contains(cookie_value)
            }
            Err(_) => false,
        };
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::api::verify_cookie;

    #[tokio::test]
    async fn test_verify_cookie() {
        let cookie: String = String::from("session=sloth101");
        let res = verify_cookie(&cookie).await;
        assert_eq!(res, true);
    }
}
