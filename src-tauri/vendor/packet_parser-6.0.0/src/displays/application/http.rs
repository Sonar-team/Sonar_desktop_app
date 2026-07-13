// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::fmt::{self, Display};

use crate::parse::application::protocols::http::HttpRequest;

impl Display for HttpRequest<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HTTP Request: method={}, uri={}, version={}, headers={:?}, body={}",
            self.method, self.uri, self.version, self.headers, self.body
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::application::protocols::http::HttpRequest;

    #[test]
    fn test_http_request_display() {
        let request = HttpRequest {
            method: "GET",
            uri: "/index.html",
            version: "HTTP/1.1",
            headers: vec![("Host", "www.example.com")],
            body: "",
        };

        let rendered = request.to_string();
        assert!(rendered.starts_with("HTTP Request:"));
        assert!(rendered.contains("method=GET"));
        assert!(rendered.contains("uri=/index.html"));
        assert!(rendered.contains("version=HTTP/1.1"));
        assert!(rendered.contains("Host"));
    }
}
