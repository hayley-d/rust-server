use crate::{ContentType, ErrorType, HttpCode, HttpMethod, MyDefault, Request, Response};
use argon2::password_hash::SaltString;
use argon2::PasswordHash;
use argon2::{password_hash::Salt, Argon2, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;
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
                eprintln!("Error parsing json");
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
                eprintln!("Error inserting user");
                return response.code(HttpCode::InternalServerError);
            }
        }

        response.add_header(
            String::from("Set-Cookie"),
            format!("session={}; HttpOnly", session_id),
        );

        return response;
    } else if request.uri == "/login" {
        let user: HashMap<String, String> = match serde_json::from_str(&request.body) {
            Ok(u) => u,
            Err(_) => {
                eprintln!("Error parsing json");
                return response.code(HttpCode::BadRequest);
            }
        };

        let input_username: &str = &user["username"];
        let input_password: &str = &user["password"];

        /*let cookie_header: Vec<String> = request
            .headers
            .into_iter()
            .filter(|h| h.contains("Cookie: session="))
            .collect();

        let cookie_header = match cookie_header.get(0) {
            Some(h) => h,
            None => {
                eprintln!("Error parsing cookie header");
                return response.code(HttpCode::BadRequest);
            }
        };

        let header_parts: Vec<&str> = cookie_header.split_whitespace().collect();

        let cookie_value: &str = match header_parts.get(1) {
            Some(v) => v,
            None => {
                eprintln!("Error parsing cookie header");
                return response.code(HttpCode::BadRequest);
            }
        };

        if verify_cookie(cookie_value).await {*/
        let contents: String = fs::read_to_string("static/users.txt").await.unwrap();

        let user_values: String = match contents
            .lines()
            .filter(|l| l.contains(input_username))
            .collect::<Vec<&str>>()
            .get(0)
        {
            Some(l) => l.to_string(),
            None => {
                eprintln!("User does not exist");
                return response.code(HttpCode::BadRequest);
            }
        };

        let user_values: Vec<&str> = user_values.split('|').collect();

        if user_values.len() != 3 {
            eprintln!("User does not exist");
            return response.code(HttpCode::BadRequest);
        }

        if user_values[0] == input_username {
            match validate_password(input_password, user_values[1]) {
                Ok(v) if v == true => (),
                Ok(_) => {
                    eprintln!("Incorrect Password");
                    return response.code(HttpCode::BadRequest);
                }
                Err(_) => {
                    eprintln!("Error validating password");
                    return response.code(HttpCode::InternalServerError);
                }
            }

            response.add_header(
                String::from("Set-Cookie"),
                format!("session={}; HttpOnly", user_values[2]),
            );
            return response;
        }

        //}
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
    let password = password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = match argon2.hash_password(&password, salt.as_salt()) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Error hashing password: {:?} ", e);
            return Err(ErrorType::InternalServerError(String::from(
                "Problem occured when creating password",
            )));
        }
    };

    let mut file_input: Vec<u8> = username.into_bytes();
    file_input.push(b'|');
    file_input.extend_from_slice(hash.to_string().as_bytes());
    file_input.push(b'|');
    file_input.extend_from_slice(session.as_bytes());
    let mut file = OpenOptions::new()
        .append(true)
        .open("static/users.txt")
        .await
        .expect("cannot open file");

    match file.write(&file_input).await {
        Ok(_) => (),
        Err(_) => {
            eprintln!("Error writing to file");
            return Err(ErrorType::InternalServerError(String::from(
                "Problem occured when writing user to db",
            )));
        }
    };

    Ok(())
}

fn validate_password(password: &str, hashed_password: &str) -> Result<bool, ErrorType> {
    let argon2 = Argon2::default();

    // Parse the hashed password
    let parsed_hash = PasswordHash::new(hashed_password).map_err(|e| {
        eprintln!("Error parsing hashed password: {:?}", e);
        ErrorType::InternalServerError(String::from(
            "Problem occurred when validating the password",
        ))
    })?;

    // Verify the password against the hashed password
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => {
            eprintln!("Incorrect password");
            return Err(ErrorType::BadRequest(String::from("Incorrect Password")));
        }
    }
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
                f.contains(cookie_value)
            }
            Err(_) => false,
        };
    }
    false
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::api::{handle_post, verify_cookie};
    use crate::{HttpCode, HttpMethod, Request, Response};

    #[tokio::test]
    async fn test_verify_cookie() {
        let cookie: String = String::from("session=sloth101");
        let res = verify_cookie(&cookie).await;
        assert_eq!(res, true);
    }

    #[tokio::test]
    async fn test_signup() {
        let request_body = json!({
            "username": "hayley",
            "password": "password"
        })
        .to_string();

        let request = Request {
            headers: Vec::new(),
            body: request_body,
            method: HttpMethod::POST,
            uri: "/signup".to_string(),
        };

        let response: Response = handle_post(request).await;
        assert_eq!(response.code, HttpCode::Ok);
    }

    #[tokio::test]
    async fn test_login() {
        let request_body = json!({
            "username": "hayley",
            "password": "password"
        })
        .to_string();

        let request = Request {
            headers: Vec::new(),
            body: request_body,
            method: HttpMethod::POST,
            uri: "/login".to_string(),
        };

        let response: Response = handle_post(request).await;
        assert_eq!(response.code, HttpCode::Ok);
    }
}
