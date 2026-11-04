//! Response Module
//!
//! Provides the Response struct similar to Express's res object.

use bytes::Bytes;
use http_body_util::Full;
use hyper::{header, HeaderMap, StatusCode};
use serde::Serialize;

/// Response struct similar to Express's res object
pub struct Response {
    status: StatusCode,
    headers: HeaderMap,
    body: Bytes,
}

impl Response {
    /// Create a new Response with default values
    pub fn new() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: Bytes::new(),
        }
    }

    /// Set the HTTP status code
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyx::Response;
    ///
    /// let res = Response::new().status(201);
    /// ```
    pub fn status(mut self, code: u16) -> Self {
        self.status = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        self
    }

    /// Set a response header
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyx::Response;
    ///
    /// let res = Response::new()
    ///     .header("X-Custom-Header", "value");
    /// ```
    pub fn header(mut self, name: &str, value: &str) -> Self {
        if let Ok(name) = header::HeaderName::from_bytes(name.as_bytes()) {
            if let Ok(value) = header::HeaderValue::from_str(value) {
                self.headers.insert(name, value);
            }
        }
        self
    }

    /// Set the Content-Type header
    pub fn content_type(self, content_type: &str) -> Self {
        self.header("content-type", content_type)
    }

    /// Send a plain text response
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyx::Response;
    ///
    /// let res = Response::new().send("Hello, World!");
    /// ```
    pub fn send(mut self, body: impl Into<String>) -> Self {
        let body_string = body.into();
        self.body = Bytes::from(body_string);
        if !self.headers.contains_key(header::CONTENT_TYPE) {
            self = self.content_type("text/plain; charset=utf-8");
        }
        self
    }

    /// Send raw bytes as response body
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use rustyx::Response;
    ///
    /// let res = Response::new()
    ///     .header("Content-Type", "image/png")
    ///     .send_bytes(image_bytes);
    /// ```
    pub fn send_bytes(mut self, body: Vec<u8>) -> Self {
        self.body = Bytes::from(body);
        self
    }

    /// Send a JSON response
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyx::Response;
    /// use serde_json::json;
    ///
    /// let res = Response::new().json(json!({ "message": "Hello" }));
    /// ```
    pub fn json<T: Serialize>(mut self, data: T) -> Self {
        match serde_json::to_vec(&data) {
            Ok(json_bytes) => {
                self.body = Bytes::from(json_bytes);
                self = self.content_type("application/json; charset=utf-8");
            }
            Err(e) => {
                self.status = StatusCode::INTERNAL_SERVER_ERROR;
                self.body = Bytes::from(format!(r#"{{"error":"Serialization error: {}"}}"#, e));
                self = self.content_type("application/json; charset=utf-8");
            }
        }
        self
    }

    /// Send an HTML response
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyx::Response;
    ///
    /// let res = Response::new().html("<h1>Hello, World!</h1>");
    /// ```
    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.body = Bytes::from(html.into());
        self.content_type("text/html; charset=utf-8")
    }

    /// Send a redirect response
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyx::Response;
    ///
    /// let res = Response::new().redirect("/login");
    /// ```
    pub fn redirect(self, url: &str) -> Self {
        self.status(302).header("location", url)
    }

    /// Send a permanent redirect response (301)
    pub fn redirect_permanent(self, url: &str) -> Self {
        self.status(301).header("location", url)
    }

    /// Send a 404 Not Found response
    pub fn not_found(self) -> Self {
        self.status(404)
            .json(serde_json::json!({ "error": "Not Found" }))
    }

    /// Send a 400 Bad Request response
    pub fn bad_request(self, message: &str) -> Self {
        self.status(400)
            .json(serde_json::json!({ "error": message }))
    }

    /// Send a 401 Unauthorized response
    pub fn unauthorized(self) -> Self {
        self.status(401)
            .json(serde_json::json!({ "error": "Unauthorized" }))
    }

    /// Send a 403 Forbidden response
    pub fn forbidden(self) -> Self {
        self.status(403)
            .json(serde_json::json!({ "error": "Forbidden" }))
    }

    /// Send a 500 Internal Server Error response
    pub fn internal_error(self, message: &str) -> Self {
        self.status(500)
            .json(serde_json::json!({ "error": message }))
    }

    /// Send a 201 Created response with JSON data
    pub fn created<T: Serialize>(self, data: T) -> Self {
        self.status(201).json(data)
    }

    /// Send a 204 No Content response
    pub fn no_content(mut self) -> Self {
        self.status = StatusCode::NO_CONTENT;
        self.body = Bytes::new();
        self
    }

    /// Set CORS headers for the response
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyx::Response;
    ///
    /// let res = Response::new()
    ///     .cors("*")
    ///     .json(serde_json::json!({ "data": "value" }));
    /// ```
    pub fn cors(self, origin: &str) -> Self {
        self.header("access-control-allow-origin", origin)
            .header(
                "access-control-allow-methods",
                "GET, POST, PUT, DELETE, PATCH, OPTIONS",
            )
            .header(
                "access-control-allow-headers",
                "Content-Type, Authorization",
            )
    }

    /// Set a cookie
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyx::Response;
    ///
    /// let res = Response::new()
    ///     .cookie("session", "abc123", CookieOptions::default());
    /// ```
    pub fn cookie(self, name: &str, value: &str, options: CookieOptions) -> Self {
        let mut cookie = format!("{}={}", name, value);

        if let Some(max_age) = options.max_age {
            cookie.push_str(&format!("; Max-Age={}", max_age));
        }
        if let Some(ref path) = options.path {
            cookie.push_str(&format!("; Path={}", path));
        }
        if let Some(ref domain) = options.domain {
            cookie.push_str(&format!("; Domain={}", domain));
        }
        if options.secure {
            cookie.push_str("; Secure");
        }
        if options.http_only {
            cookie.push_str("; HttpOnly");
        }
        if let Some(ref same_site) = options.same_site {
            cookie.push_str(&format!("; SameSite={}", same_site));
        }

        self.header("set-cookie", &cookie)
    }

    /// Clear a cookie
    pub fn clear_cookie(self, name: &str) -> Self {
        self.cookie(
            name,
            "",
            CookieOptions {
                max_age: Some(0),
                ..Default::default()
            },
        )
    }

    /// Convert to hyper Response
    pub fn into_hyper(self) -> hyper::Response<Full<Bytes>> {
        let mut response = hyper::Response::builder().status(self.status);

        for (name, value) in self.headers.iter() {
            response = response.header(name, value);
        }

        response.body(Full::new(self.body)).unwrap()
    }

    /// Get the current status code
    pub fn get_status(&self) -> StatusCode {
        self.status
    }

    /// Get the current headers
    pub fn get_headers(&self) -> &HeaderMap {
        &self.headers
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

/// Cookie options for setting cookies
#[derive(Debug, Clone, Default)]
pub struct CookieOptions {
    pub max_age: Option<i64>,
    pub path: Option<String>,
    pub domain: Option<String>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
}

impl CookieOptions {
    /// Create a new CookieOptions with sensible defaults
    pub fn new() -> Self {
        Self {
            max_age: None,
            path: Some("/".to_string()),
            domain: None,
            secure: false,
            http_only: true,
            same_site: Some("Lax".to_string()),
        }
    }

    /// Set the max age in seconds
    pub fn max_age(mut self, seconds: i64) -> Self {
        self.max_age = Some(seconds);
        self
    }

    /// Set the cookie path
    pub fn path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }

    /// Set the cookie domain
    pub fn domain(mut self, domain: &str) -> Self {
        self.domain = Some(domain.to_string());
        self
    }

    /// Set the secure flag
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Set the HttpOnly flag
    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    /// Set the SameSite attribute
    pub fn same_site(mut self, same_site: &str) -> Self {
        self.same_site = Some(same_site.to_string());
        self
    }
}
