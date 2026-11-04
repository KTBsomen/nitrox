//! Request Module
//!
//! Provides the Request struct similar to Express's req object.

use crate::error::{Error, Result};

use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::{header::HeaderValue, HeaderMap, Method, Uri, Version};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::net::SocketAddr;

/// Request struct similar to Express's req object
#[derive(Debug)]
pub struct Request {
    method: Method,
    uri: Uri,
    version: Version,
    headers: HeaderMap,
    body: Bytes,
    params: HashMap<String, String>,
    query: HashMap<String, String>,
    remote_addr: SocketAddr,
}

impl Request {
    /// Create a new Request from a hyper request
    pub async fn from_hyper(
        req: hyper::Request<Incoming>,
        remote_addr: SocketAddr,
    ) -> Result<Self> {
        let (parts, body) = req.into_parts();

        // Parse query string
        let query = parts
            .uri
            .query()
            .map(|q| {
                url::form_urlencoded::parse(q.as_bytes())
                    .into_owned()
                    .collect()
            })
            .unwrap_or_default();

        // Collect body bytes
        let body_bytes = body
            .collect()
            .await
            .map_err(|e| Error::Internal(e.to_string()))?
            .to_bytes();

        Ok(Self {
            method: parts.method,
            uri: parts.uri,
            version: parts.version,
            headers: parts.headers,
            body: body_bytes,
            params: HashMap::new(),
            query,
            remote_addr,
        })
    }

    /// Get the HTTP method
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Get the request path
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    /// Get the full URI
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Get the HTTP version
    pub fn version(&self) -> Version {
        self.version
    }

    /// Get the request headers
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get a specific header value
    pub fn get_header(&self, name: &str) -> Option<&HeaderValue> {
        self.headers.get(name)
    }

    /// Get a header value as a string
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }

    /// Get the raw body bytes
    pub fn body(&self) -> &Bytes {
        &self.body
    }

    /// Get the body as a string
    pub fn body_string(&self) -> Result<String> {
        String::from_utf8(self.body.to_vec())
            .map_err(|_| Error::ParseError("Invalid UTF-8 in body".to_string()))
    }

    /// Parse the body as JSON
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[derive(Deserialize)]
    /// struct CreateUser {
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// let user: CreateUser = req.json()?;
    /// ```
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.body)
            .map_err(|e| Error::ParseError(format!("JSON parse error: {}", e)))
    }

    /// Get route parameters
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // For route "/users/:id"
    /// let id = req.params().get("id");
    /// ```
    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    /// Get a specific route parameter
    pub fn param(&self, name: &str) -> Option<&String> {
        self.params.get(name)
    }

    /// Set route parameters (called internally by the router)
    pub fn set_params(&mut self, params: HashMap<String, String>) {
        self.params = params;
    }

    /// Get query parameters
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // For URL "/users?page=1&limit=10"
    /// let page = req.query().get("page");
    /// ```
    pub fn query(&self) -> &HashMap<String, String> {
        &self.query
    }

    /// Get a specific query parameter
    pub fn query_param(&self, name: &str) -> Option<&String> {
        self.query.get(name)
    }

    /// Get the remote address of the client
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Get the client's IP address
    pub fn ip(&self) -> std::net::IpAddr {
        self.remote_addr.ip()
    }

    /// Check if the request accepts a specific content type
    pub fn accepts(&self, content_type: &str) -> bool {
        self.header("accept")
            .map(|accept| accept.contains(content_type))
            .unwrap_or(false)
    }

    /// Get the Content-Type header
    pub fn content_type(&self) -> Option<&str> {
        self.header("content-type")
    }

    /// Check if the request is JSON
    pub fn is_json(&self) -> bool {
        self.content_type()
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false)
    }

    /// Get the Content-Length
    pub fn content_length(&self) -> Option<usize> {
        self.header("content-length").and_then(|v| v.parse().ok())
    }

    /// Get the Host header
    pub fn host(&self) -> Option<&str> {
        self.header("host")
    }

    /// Get the User-Agent header
    pub fn user_agent(&self) -> Option<&str> {
        self.header("user-agent")
    }

    /// Check if the request is a XHR request
    pub fn is_xhr(&self) -> bool {
        self.header("x-requested-with")
            .map(|v| v.eq_ignore_ascii_case("xmlhttprequest"))
            .unwrap_or(false)
    }

    /// Get the Authorization header
    pub fn authorization(&self) -> Option<&str> {
        self.header("authorization")
    }

    /// Extract Bearer token from Authorization header
    pub fn bearer_token(&self) -> Option<&str> {
        self.authorization()
            .filter(|auth| auth.starts_with("Bearer "))
            .map(|auth| &auth[7..])
    }
}
