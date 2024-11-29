pub mod request_validation {
    use crate::ErrorType;

    pub fn handle_request(buffer: &[u8]) -> Result<(), ErrorType> {
        // convert into a utf8 string
        let request = match String::from_utf8(buffer.to_vec()) {
            Ok(r) => r,
            Err(_) => {
                let error: ErrorType = ErrorType::BadRequest(String::from("Invalid UTF-8 request"));
                return Err(error);
            }
        };

        // Split request into lines
        let mut request = request.lines();

        let request_line: &str = match request.next() {
            Some(l) => l,
            None => {
                let error: ErrorType = ErrorType::BadRequest(String::from("Missing request line"));
                return Err(error);
            }
        };

        request_line_validation(request_line)?;
        validate_headers(request.clone())?;
        check_overflow(buffer)?;

        return Ok(());
    }

    fn validate_headers<'a>(lines: impl Iterator<Item = &'a str>) -> Result<(), ErrorType> {
        let mut host_count: u8 = 0;

        for line in lines {
            if line.is_empty() {
                break;
            }

            if line.starts_with("Host:") {
                host_count += 1;
            }
        }

        if host_count != 1 {
            let error: ErrorType =
                ErrorType::BadRequest(format!("Invalid host count: {}", host_count));
            return Err(error);
        }

        return Ok(());
    }

    fn validate_uri(uri: &str) -> Result<(), ErrorType> {
        if uri.is_empty()
            || uri.contains("..")
            || uri.starts_with("http://")
            || !uri.starts_with('/')
            || uri.contains('\0')
        {
            let error: ErrorType = ErrorType::BadRequest(format!("Invalid uri: {}", uri));
            return Err(error);
        }

        if uri.chars().any(|c| c.is_control()) {
            let error: ErrorType = ErrorType::BadRequest(format!("Invalid uri: {}", uri));
            return Err(error);
        }

        let forbidden_characters = ['<', '>', '{', '}', '|', '\\', '^', '`', '[', ']', ' ', '%'];

        if uri.chars().any(|c| forbidden_characters.contains(&c)) {
            let error: ErrorType = ErrorType::BadRequest(format!("Invalid uri: {}", uri));
            return Err(error);
        }

        let forbidden_words: [&str; 13] = [
            "rm", "sh", "bash", "python", "perl", "exec", "delete", "drop", "cmd", "script", ";--",
            "' OR '", "&&",
        ];

        if forbidden_words.iter().any(|w| uri.contains(w)) {
            let error: ErrorType = ErrorType::BadRequest(format!("Invalid uri: {}", uri));
            return Err(error);
        }

        if uri.chars().any(|c| forbidden_characters.contains(&c)) {
            let error: ErrorType = ErrorType::BadRequest(format!("Invalid uri: {}", uri));
            return Err(error);
        }

        return Ok(());
    }

    fn request_line_validation(request_line: &str) -> Result<(), ErrorType> {
        let request_line_parts: Vec<&str> = request_line.split_whitespace().collect();

        // validate method, URI and  protocol
        if request_line_parts.len() != 3 {
            let error: ErrorType = ErrorType::BadRequest(String::from("Invalid request line"));
            return Err(error);
        }

        let (method, uri, protocol): (&str, &str, &str) = (
            request_line_parts[0],
            request_line_parts[1],
            request_line_parts[2],
        );

        if !["GET", "POST", "PUT", "DELETE"].contains(&method) {
            let error: ErrorType = ErrorType::BadRequest(String::from("Invalid request method"));
            return Err(error);
        }

        if !uri.starts_with('/') {
            let error: ErrorType = ErrorType::BadRequest(String::from("Invalid request uri"));
            return Err(error);
        }

        validate_uri(uri)?;

        if !["HTTP/1.1", "HTTP/1.0", "HTTP/2", "HTTP/3"].contains(&protocol) {
            let error: ErrorType = ErrorType::BadRequest(String::from("Invalid request protocol"));
            return Err(error);
        }

        return Ok(());
    }

    fn check_overflow(data: &[u8]) -> Result<(), ErrorType> {
        if &data[data.len() - 4..] != b"\r\n\r\n" {
            let error: ErrorType = ErrorType::BadRequest(String::from("Request overflow"));
            return Err(error);
        }
        return Ok(());
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::ErrorType;
        #[test]
        fn test_handle_request_valid() {
            let valid_buffer: &[u8] = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";

            eprintln!("{:?}", handle_request(valid_buffer));
            // Should pass without errors
            assert!(handle_request(valid_buffer).is_ok());
        }

        #[test]
        fn test_handle_request_invalid_utf8() {
            let invalid_utf8: &[u8] = &[0x80, 0x81, 0x82, 0x83];

            let result: ErrorType = match handle_request(invalid_utf8) {
                Ok(_) => ErrorType::InternalServerError(String::from("Wrong result")),
                Err(e) => e,
            };
            // Should return a BadRequest error due to invalid UTF-8
            assert_eq!(
                result,
                ErrorType::BadRequest(String::from("Invalid UTF-8 request"))
            );
        }

        #[test]
        fn test_validate_headers_missing_host() {
            let headers = vec!["GET / HTTP/1.1", "User-Agent: test"];

            let result: ErrorType = match validate_headers(headers.iter().copied()) {
                Ok(_) => ErrorType::InternalServerError(String::from("Wrong result")),
                Err(e) => e,
            };

            // Should return a BadRequest error since "Host" header is missing
            assert_eq!(
                result,
                ErrorType::BadRequest(String::from("Invalid host count: 0"))
            );
        }

        #[test]
        fn test_validate_headers_multiple_host() {
            let headers = vec!["Host: example.com", "Host: another.com", "User-Agent: test"];

            let result: ErrorType = match validate_headers(headers.iter().copied()) {
                Ok(_) => ErrorType::InternalServerError(String::from("Wrong result")),
                Err(e) => e,
            };

            // Should return a BadRequest error since there are multiple "Host" headers
            assert_eq!(
                result,
                ErrorType::BadRequest(String::from("Invalid host count: 2"))
            );
        }

        #[test]
        fn test_validate_uri_valid() {
            // Valid URI
            let valid_uri = "/index.html";

            // Should pass without errors
            assert!(validate_uri(valid_uri).is_ok());
        }

        #[test]
        fn test_validate_uri_invalid_path_traversal() {
            let invalid_uri = "/../../etc/passwd";

            let result: ErrorType = match validate_uri(invalid_uri) {
                Ok(_) => ErrorType::InternalServerError(String::from("Wrong result")),
                Err(e) => e,
            };

            assert_eq!(
                result,
                ErrorType::BadRequest(String::from("Invalid URI: /../../etc/passwd"))
            );
        }

        #[test]
        fn test_validate_uri_invalid_forbidden_chars() {
            let invalid_uri = "/index.html?<script>alert(1)</script>";

            let result: ErrorType = match validate_uri(invalid_uri) {
                Ok(_) => ErrorType::InternalServerError(String::from("Wrong result")),
                Err(e) => e,
            };

            assert_eq!(
                result,
                ErrorType::BadRequest(String::from(
                    "Invalid URI: /index.html?<script>alert(1)</script>"
                ))
            );
        }

        #[test]
        fn test_request_line_validation_valid() {
            let valid_line = "GET /index.html HTTP/1.1";

            // Should pass without errors
            assert!(request_line_validation(valid_line).is_ok());
        }

        #[test]
        fn test_request_line_validation_invalid_method() {
            let invalid_line = "INVALID /index.html HTTP/1.1";

            let result: ErrorType = match request_line_validation(invalid_line) {
                Ok(_) => ErrorType::InternalServerError(String::from("Wrong result")),
                Err(e) => e,
            };

            assert_eq!(
                result,
                ErrorType::BadRequest(String::from("Invalid request method"))
            );
        }

        #[test]
        fn test_request_line_validation_invalid_protocol() {
            let invalid_line = "GET /index.html HTTP/2.0";

            let result: ErrorType = match request_line_validation(invalid_line) {
                Ok(_) => ErrorType::InternalServerError(String::from("Wrong result")),
                Err(e) => e,
            };

            assert_eq!(
                result,
                ErrorType::BadRequest(String::from("Invalid request protocol"))
            );
        }

        #[test]
        fn test_check_overflow_valid() {
            let valid_data: &[u8] = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
            eprintln!("{:?}", check_overflow(valid_data));
            assert!(check_overflow(valid_data).is_ok());
        }

        #[test]
        fn test_check_overflow_invalid() {
            let invalid_data: &[u8] = b"GET / HTTP/1.1\r\nHost: example.com";

            let result: ErrorType = match check_overflow(invalid_data) {
                Ok(_) => ErrorType::InternalServerError(String::from("Wrong result")),
                Err(e) => e,
            };

            assert_eq!(
                result,
                ErrorType::BadRequest(String::from("Request overflow"))
            );
        }
    }
}
