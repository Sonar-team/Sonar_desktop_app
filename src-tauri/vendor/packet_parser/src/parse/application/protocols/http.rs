//! Module for parsing HTTP packets.

use std::fmt;

/// The `HttpRequest` struct represents a parsed HTTP request.
#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub uri: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl fmt::Display for HttpRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HTTP Request: method={}, uri={}, version={}, headers={:?}, body={}",
            self.method, self.uri, self.version, self.headers, self.body
        )
    }
}

/// Parses an HTTP request from a given payload.
///
/// # Arguments
///
/// * `payload` - A byte slice representing the raw HTTP packet data.
///
/// # Returns
///
/// * `Result<HttpRequest, bool>` - Returns `Ok(HttpRequest)` if parsing is successful,
///   otherwise returns `Err(false)` indicating an invalid HTTP request.
pub fn parse_http_request(payload: &[u8]) -> Result<HttpRequest, bool> {
    let payload_str = match std::str::from_utf8(payload) {
        Ok(s) => s,
        Err(_) => return Err(false),
    };

    let mut lines = payload_str.split("\r\n");

    let request_line = match lines.next() {
        Some(line) => line,
        None => return Err(false),
    };

    let mut request_parts = request_line.split_whitespace();
    let method = match request_parts.next() {
        Some(part) => part.to_string(),
        None => return Err(false),
    };
    let uri = match request_parts.next() {
        Some(part) => part.to_string(),
        None => return Err(false),
    };
    let version = match request_parts.next() {
        Some(part) => part.to_string(),
        None => return Err(false),
    };

    let mut headers = Vec::new();
    for line in lines.by_ref() {
        if line.is_empty() {
            break;
        }
        let mut header_parts = line.splitn(2, ':');
        let name = match header_parts.next() {
            Some(part) => part.trim().to_string(),
            None => return Err(false),
        };
        let value = match header_parts.next() {
            Some(part) => part.trim().to_string(),
            None => return Err(false),
        };
        headers.push((name, value));
    }

    let body = lines.collect::<Vec<&str>>().join("\r\n");

    Ok(HttpRequest {
        method,
        uri,
        version,
        headers,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_request() {
        let http_payload = b"GET /index.html HTTP/1.1\r\nHost: www.example.com\r\nUser-Agent: curl/7.68.0\r\nAccept: */*\r\n\r\n";
        match parse_http_request(http_payload) {
            Ok(request) => {
                assert_eq!(request.method, "GET");
                assert_eq!(request.uri, "/index.html");
                assert_eq!(request.version, "HTTP/1.1");
                assert_eq!(request.headers.len(), 3);
                assert_eq!(
                    request.headers[0],
                    ("Host".to_string(), "www.example.com".to_string())
                );
                assert_eq!(
                    request.headers[1],
                    ("User-Agent".to_string(), "curl/7.68.0".to_string())
                );
                assert_eq!(
                    request.headers[2],
                    ("Accept".to_string(), "*/*".to_string())
                );
                assert_eq!(request.body, "");
            }
            Err(_) => panic!("Expected HTTP request"),
        }
    }

    #[test]
    fn test_parse_http_request_with_body() {
        let http_payload = b"POST /submit HTTP/1.1\r\nHost: www.example.com\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 13\r\n\r\nfield1=value1";
        match parse_http_request(http_payload) {
            Ok(request) => {
                assert_eq!(request.method, "POST");
                assert_eq!(request.uri, "/submit");
                assert_eq!(request.version, "HTTP/1.1");
                assert_eq!(request.headers.len(), 3);
                assert_eq!(
                    request.headers[0],
                    ("Host".to_string(), "www.example.com".to_string())
                );
                assert_eq!(
                    request.headers[1],
                    (
                        "Content-Type".to_string(),
                        "application/x-www-form-urlencoded".to_string()
                    )
                );
                assert_eq!(
                    request.headers[2],
                    ("Content-Length".to_string(), "13".to_string())
                );
                assert_eq!(request.body, "field1=value1");
            }
            Err(_) => panic!("Expected HTTP request with body"),
        }
    }

    #[test]
    fn test_parse_http_request_invalid() {
        let http_payload = b"INVALID REQUEST\r\n\r\n";
        match parse_http_request(http_payload) {
            Ok(_) => panic!("Expected invalid HTTP request"),
            Err(is_http) => assert!(!is_http),
        }
    }
}
