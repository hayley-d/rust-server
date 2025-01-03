use crate::{ContentType, ErrorType, HttpCode, HttpMethod, Logger, MyDefault, Request, Response};
use argon2::password_hash::SaltString;
use argon2::PasswordHash;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use colored::Colorize;
use log::info;
use rand::rngs::OsRng;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

/// Reads the contents of a file asynchronously and returns its data as bytes.
///
/// # Arguments
/// - `path`: A string slice representing the file path.
///
/// # Returns
/// A `Vec<u8>` containing the file's contents.
///
/// # Panics
/// Panics if the file cannot be accessed, or if the read operation fails.
pub async fn read_file_to_bytes(path: &str) -> Vec<u8> {
    let metadata = fs::metadata(path).await.unwrap();
    let mut file = File::open(path).await.unwrap();
    let mut buffer: Vec<u8> = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut buffer).await.unwrap();
    return buffer;
}

/// Handles incoming HTTP requests by delegating to specific handlers
/// based on the HTTP method.
///
/// # Arguments
/// - `request`: The HTTP request to be processed.
/// - `logger`: A thread-safe `Logger` instance for recording log entries.
///
/// # Returns
/// A `Response` that corresponds to the processed request.
pub async fn handle_response(request: Request, logger: Arc<Mutex<Logger>>) -> Response {
    match request.method {
        HttpMethod::GET => handle_get(request, logger).await,
        HttpMethod::POST => handle_post(request, logger).await,
        HttpMethod::PUT => handle_put(request, logger).await,
        HttpMethod::PATCH => handle_patch(request, logger).await,
        HttpMethod::DELETE => handle_delete(request, logger).await,
    }
}

/// Processes HTTP GET requests and returns an appropriate response.
///
/// # Arguments
/// - `request`: The HTTP GET request to process.
/// - `logger`: A shared logger instance to track activity.
///
/// # Returns
/// A `Response` specific to the GET request, such as HTML content
/// or an error response if applicable.
async fn handle_get(request: Request, logger: Arc<Mutex<Logger>>) -> Response {
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
        info!("GET / from {}");
        response.add_body(read_file_to_bytes("static/index.html").await);
    } else if request.uri == "/hayley" {
        thread::sleep(Duration::from_secs(5));

        response.add_body(read_file_to_bytes("static/index.html").await);
    } else if request.uri == "/home" {
        response.add_body(read_file_to_bytes("static/home.html").await);
    } else {
        // Error
        println!(
            "{} {} {} {}",
            ">>".red().bold(),
            "No matching routes for".red(),
            request.method.to_string().magenta(),
            request.uri.cyan()
        );
        return response
            .body(String::from("404: Invalid route").into())
            .content_type(ContentType::Text)
            .code(HttpCode::BadRequest);
    }
    return response;
}

/// Handles HTTP POST requests for various operations, such as user
/// signup and login.
///
/// # Arguments
/// - `request`: The HTTP POST request containing the payload.
/// - `logger`: A thread-safe logger to capture logs during processing.
///
/// # Returns
/// A `Response` corresponding to the POST request, with outcomes like
/// successful account creation, login confirmation, or error handling.
async fn handle_post(request: Request, logger: Arc<Mutex<Logger>>) -> Response {
    let mut response = Response::default()
        .await
        .compression(request.is_compression_supported())
        .body(read_file_to_bytes("static/index.html").await)
        .content_type(ContentType::Text);

    if request.uri == "/signup" {
        // parse the JSON into a hashmap
        let user: HashMap<String, String> = match serde_json::from_str(&request.body) {
            Ok(u) => u,
            Err(_) => {
                let error = ErrorType::BadRequest(String::from("Invalid JSON request."));
                logger.lock().await.log_error(&error);
                println!(
                    "{} {} {} {}",
                    ">>".red().bold(),
                    "Invalid JSON for".red(),
                    request.method.to_string().magenta(),
                    request.uri.cyan()
                );
                return response
                    .body(String::from("Invalid JSON.").into())
                    .code(HttpCode::BadRequest);
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
                let error = ErrorType::InternalServerError(String::from(
                    "Problem when attempting to insert new user.",
                ));
                logger.lock().await.log_error(&error);
                println!(
                    "{} {} {} {}",
                    ">>".red().bold(),
                    "Error while creating new user ".red(),
                    request.method.to_string().magenta(),
                    request.uri.cyan()
                );
                return response
                    .body(String::from("Problem occured when attempting to add new user.").into())
                    .code(HttpCode::InternalServerError);
            }
        }

        response.add_header(
            String::from("Set-Cookie"),
            format!("session={}; HttpOnly", session_id),
        );

        return response
            .body(String::from("New user successfully created!").into())
            .code(HttpCode::Ok);
    } else if request.uri == "/login" {
        let user: HashMap<String, String> = match serde_json::from_str(&request.body) {
            Ok(u) => u,
            Err(_) => {
                let error = ErrorType::BadRequest(String::from("Invalid JSON request."));
                logger.lock().await.log_error(&error);
                println!(
                    "{} {} {} {}",
                    ">>".red().bold(),
                    "Invaid JSON for".red(),
                    request.method.to_string().magenta(),
                    request.uri.cyan()
                );
                return response
                    .body(String::from("Invalid JSON.").into())
                    .code(HttpCode::BadRequest)
                    .content_type(ContentType::Text);
            }
        };

        let input_username: &str = &user["username"];
        let input_password: &str = &user["password"];

        let contents: String = fs::read_to_string("static/users.txt").await.unwrap();

        let user_values: String = match contents
            .lines()
            .filter(|l| l.contains(input_username))
            .collect::<Vec<&str>>()
            .get(0)
        {
            Some(l) => l.to_string(),
            None => {
                let error = ErrorType::BadRequest(String::from(
                    "Attempt to login to a user account that does not exist",
                ));
                logger.lock().await.log_error(&error);
                println!(
                    "{} {} {} {}",
                    ">>".red().bold(),
                    "User not found for".red(),
                    request.method.to_string().magenta(),
                    request.uri.cyan()
                );
                return response
                    .body(String::from("No user exists with the provided details.").into())
                    .code(HttpCode::BadRequest)
                    .content_type(ContentType::Text);
            }
        };

        let user_values: Vec<&str> = user_values.split('|').collect();

        if user_values.len() != 3 {
            let error = ErrorType::BadRequest(String::from(
                "Attempt to login to a user account that does not exist",
            ));
            logger.lock().await.log_error(&error);
            return response
                .body(String::from("No user exists with the provided details.").into())
                .code(HttpCode::BadRequest);
        }

        if user_values[0] == input_username {
            match validate_password(input_password, user_values[1]) {
                Ok(v) if v == true => (),
                Ok(_) => {
                    let error = ErrorType::BadRequest(String::from(
                        "Attempt to login with incorrect password.",
                    ));
                    logger.lock().await.log_error(&error);
                    println!(
                        "{} {} {}",
                        ">>".red().bold(),
                        "Attempt to login with incorrect password for".red(),
                        input_username.cyan()
                    );
                    return response
                        .body(String::from("Incorrect Password.").into())
                        .code(HttpCode::BadRequest);
                }
                Err(_) => {
                    let error = ErrorType::InternalServerError(String::from(
                        "Problem when validating password.",
                    ));
                    logger.lock().await.log_error(&error);
                    return response
                        .body(String::from("Problem occured when validating password.").into())
                        .code(HttpCode::InternalServerError);
                }
            }

            response.add_header(
                String::from("Set-Cookie"),
                format!("session={}; HttpOnly", user_values[2]),
            );

            return response
                .body(String::from("Authentification successful!").into())
                .code(HttpCode::Ok);
        }

        //}
    }
    let error = ErrorType::BadRequest(String::from("Invalid post request."));
    logger.lock().await.log_error(&error);
    return response
        .body(String::from("Invalid post URI.").into())
        .code(HttpCode::BadRequest);
}

/// Handles HTTP PUT requests. Currently, it responds with a `MethodNotAllowed`
/// HTTP status code, as this method is not implemented.
///
/// # Arguments
/// - `request`: The HTTP PUT request to process.
/// - `logger`: A shared logger instance for recording log entries.
///
/// # Returns
/// A `Response` with a `MethodNotAllowed` status.
async fn handle_put(request: Request, logger: Arc<Mutex<Logger>>) -> Response {
    let response = Response::default()
        .await
        .compression(request.is_compression_supported())
        .body(read_file_to_bytes("static/index.html").await)
        .code(HttpCode::MethodNotAllowed);

    return response;
}

/// Handles HTTP PATCH requests. Currently, it responds with a `MethodNotAllowed`
/// HTTP status code, as this method is not implemented.
///
/// # Arguments
/// - `request`: The HTTP PATCH request to process.
/// - `logger`: A shared logger instance for recording log entries.
///
/// # Returns
/// A `Response` with a `MethodNotAllowed` status.
async fn handle_patch(request: Request, logger: Arc<Mutex<Logger>>) -> Response {
    let response = Response::default()
        .await
        .compression(request.is_compression_supported())
        .body(read_file_to_bytes("static/index.html").await)
        .code(HttpCode::MethodNotAllowed);

    return response;
}

/// Processes HTTP DELETE requests to remove specified files if the user
/// is authenticated via a valid session cookie.
///
/// # Arguments
/// - `request`: The HTTP DELETE request, containing the file information and session.
/// - `logger`: A thread-safe logger to track errors and actions.
///
/// # Returns
/// A `Response` indicating success or failure of the file deletion process.
async fn handle_delete(request: Request, logger: Arc<Mutex<Logger>>) -> Response {
    let response = Response::default()
        .await
        .compression(request.is_compression_supported())
        .body(read_file_to_bytes("static/index.html").await)
        .code(HttpCode::BadRequest)
        .content_type(ContentType::Text);

    let file: HashMap<String, String> = match serde_json::from_str(&request.body) {
        Ok(u) => u,
        Err(_) => {
            let error = ErrorType::BadRequest(String::from("Invalid JSON request."));
            logger.lock().await.log_error(&error);
            return response
                .body(String::from("Invalid JSON").into())
                .code(HttpCode::BadRequest);
        }
    };

    let file_name: &String = &file["file_name"];

    let cookie_header: Vec<String> = request
        .headers
        .into_iter()
        .filter(|h| h.contains("Cookie: session="))
        .collect();

    let cookie_header = match cookie_header.get(0) {
        Some(h) => h,
        None => {
            let error = ErrorType::BadRequest(String::from(
                "Attempt to delete without proper authentification.",
            ));
            logger.lock().await.log_error(&error);
            return response
                .body(String::from("Unable to delete file without proper authentification.").into())
                .code(HttpCode::BadRequest);
        }
    };

    let header_parts: Vec<&str> = cookie_header.split_whitespace().collect();

    let cookie_value: &str = match header_parts.get(1) {
        Some(v) => v,
        None => {
            let error = ErrorType::BadRequest(String::from(
                "Attempt to delete without proper authentification.",
            ));
            logger.lock().await.log_error(&error);
            return response
                .body(String::from("Unable to delete file without proper authentification.").into())
                .code(HttpCode::BadRequest);
        }
    };

    // cookie_value = session=sessionID
    if verify_cookie(cookie_value).await {
        // session has been verified process the delete
        match fs::remove_file(file_name).await {
            Ok(_) => {
                return response
                    .body(String::from("File successfully deleted.").into())
                    .code(HttpCode::Ok);
            }
            Err(_) => {
                let error = ErrorType::BadRequest(String::from(
                    "Attempt to remove file that does not exist",
                ));
                logger.lock().await.log_error(&error);
                return response
                    .body(String::from("Unable to delete file: File does not exist.").into())
                    .code(HttpCode::BadRequest);
            }
        }
    }

    return response
        .body(String::from("Unable to delete file.").into())
        .code(HttpCode::BadRequest);
}

/// Inserts a user into the database.
///
/// Hashes the user's password using Argon2, then appends the user's details
/// (username, hashed password, session ID) to the `users.txt` file.
///
/// # Arguments
/// - `username`: The username of the new user.
/// - `password`: The password of the new user.
/// - `session`: A unique session ID for the user.
///
/// # Returns
/// - `Ok(())` if the operation is successful.
/// - `Err(ErrorType)` if an error occurs.
async fn insert_user(username: String, password: String, session: String) -> Result<(), ErrorType> {
    let password = password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = match argon2.hash_password(&password, salt.as_salt()) {
        Ok(hash) => hash,
        Err(_) => {
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
            return Err(ErrorType::InternalServerError(String::from(
                "Problem occured when writing user to db",
            )));
        }
    };

    Ok(())
}

/// Validates a password against a hashed password.
///
/// Uses Argon2 to verify if the provided password matches the stored hash.
///
/// # Arguments
/// - `password`: The plaintext password provided by the user.
/// - `hashed_password`: The stored hashed password.
///
/// # Returns
/// - `Ok(true)` if the password matches the hash.
/// - `Ok(false)` if the password does not match the hash.
/// - `Err(ErrorType)` if a validation error occurs.
fn validate_password(password: &str, hashed_password: &str) -> Result<bool, ErrorType> {
    let argon2 = Argon2::default();

    // Parse the hashed password
    let parsed_hash = PasswordHash::new(hashed_password).map_err(|_| {
        ErrorType::InternalServerError(String::from(
            "Problem occurred when validating the password",
        ))
    })?;

    // Verify the password against the hashed password
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => {
            return Err(ErrorType::BadRequest(String::from("Incorrect Password")));
        }
    }
}

/// Generates a random session ID.
///
/// # Returns
/// - A `String` representing the session ID.
fn generate_session_id() -> String {
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect()
}

/// Verifies the session cookie.
///
/// Reads the `users.txt` file to check if the provided session cookie matches an active session.
///
/// # Arguments
/// - `cookie`: The session cookie to verify.
///
/// # Returns
/// - `true` if the session is valid.
/// - `false` otherwise.
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

    use std::sync::Arc;

    use serde_json::json;
    use tokio::sync::Mutex;

    use crate::api::{handle_post, verify_cookie};
    use crate::{HttpCode, HttpMethod, Logger, Request, Response};

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
        let logger: Arc<Mutex<Logger>> = Arc::new(Mutex::new(Logger::new("server.log")));
        let response: Response = handle_post(request, logger).await;
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
        let logger: Arc<Mutex<Logger>> = Arc::new(Mutex::new(Logger::new("server.log")));
        let response: Response = handle_post(request, logger).await;
        assert_eq!(response.code, HttpCode::Ok);
    }
}
