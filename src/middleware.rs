//! Middleware Module
//!
//! Provides middleware functionality similar to Express middleware.

pub mod rate_limit;

use crate::request::Request;
use crate::response::Response;

use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tracing::info;

// Re-export rate limiting
pub use rate_limit::{rate_limiter, simple_rate_limit, RateLimiter, RateLimiterConfig};

/// Next function type for middleware chaining
pub type Next =
    Arc<dyn Fn(Request, Response) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

/// Middleware function type
pub type MiddlewareFn = Box<
    dyn Fn(Request, Response, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync,
>;

/// Trait for implementing middleware
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Process the request and optionally call the next middleware
    async fn handle(&self, req: Request, res: Response, next: Next) -> Response;
}

/// Stack of middleware functions
pub struct MiddlewareStack {
    stack: Vec<MiddlewareFn>,
}

impl MiddlewareStack {
    /// Create a new empty middleware stack
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Push a middleware function onto the stack
    pub fn push(&mut self, middleware: MiddlewareFn) {
        self.stack.push(middleware);
    }

    /// Get the number of middleware in the stack
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

impl Default for MiddlewareStack {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Built-in Middleware
// ============================================================================

/// Logger middleware - logs all incoming requests
///
/// # Example
///
/// ```rust,ignore
/// use rustyx::middleware::logger;
///
/// app.use_middleware(logger());
/// ```
pub fn logger() -> impl Fn(Request, Response, Next) -> Pin<Box<dyn Future<Output = Response> + Send>>
       + Send
       + Sync
       + Clone {
    move |req: Request, res: Response, next: Next| {
        Box::pin(async move {
            let method = req.method().clone();
            let path = req.path().to_string();
            let start = std::time::Instant::now();

            let response = next(req, res).await;

            let duration = start.elapsed();
            let status = response.get_status();

            info!("{} {} {} - {:?}", method, path, status.as_u16(), duration);

            response
        })
    }
}

/// CORS middleware - adds CORS headers to all responses
///
/// # Example
///
/// ```rust,ignore
/// use rustyx::middleware::cors;
///
/// // Allow all origins
/// app.use_middleware(cors("*"));
///
/// // Allow specific origin
/// app.use_middleware(cors("https://example.com"));
/// ```
pub fn cors(
    origin: &'static str,
) -> impl Fn(Request, Response, Next) -> Pin<Box<dyn Future<Output = Response> + Send>>
       + Send
       + Sync
       + Clone {
    move |req: Request, res: Response, next: Next| {
        Box::pin(async move {
            // Handle preflight requests
            if req.method() == hyper::Method::OPTIONS {
                return res
                    .status(204)
                    .cors(origin)
                    .header("access-control-max-age", "86400");
            }

            let response = next(req, res).await;
            response
                .header("access-control-allow-origin", origin)
                .header(
                    "access-control-allow-methods",
                    "GET, POST, PUT, DELETE, PATCH, OPTIONS",
                )
                .header(
                    "access-control-allow-headers",
                    "Content-Type, Authorization",
                )
        })
    }
}

/// Advanced CORS options
#[derive(Clone)]
pub struct CorsOptions {
    pub origin: String,
    pub methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub credentials: bool,
    pub max_age: u32,
}

impl Default for CorsOptions {
    fn default() -> Self {
        Self {
            origin: "*".to_string(),
            methods: vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            allowed_headers: vec!["Content-Type", "Authorization"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            exposed_headers: Vec::new(),
            credentials: false,
            max_age: 86400,
        }
    }
}

impl CorsOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn origin(mut self, origin: &str) -> Self {
        self.origin = origin.to_string();
        self
    }

    pub fn credentials(mut self, allow: bool) -> Self {
        self.credentials = allow;
        self
    }
}

/// Advanced CORS middleware with options
pub fn cors_with_options(
    options: CorsOptions,
) -> impl Fn(Request, Response, Next) -> Pin<Box<dyn Future<Output = Response> + Send>>
       + Send
       + Sync
       + Clone {
    move |req: Request, res: Response, next: Next| {
        let opts = options.clone();
        Box::pin(async move {
            if req.method() == hyper::Method::OPTIONS {
                let mut response = res.status(204);
                response = response.header("access-control-allow-origin", &opts.origin);
                response =
                    response.header("access-control-allow-methods", &opts.methods.join(", "));
                response = response.header(
                    "access-control-allow-headers",
                    &opts.allowed_headers.join(", "),
                );
                if opts.credentials {
                    response = response.header("access-control-allow-credentials", "true");
                }
                response = response.header("access-control-max-age", &opts.max_age.to_string());
                return response;
            }

            let response = next(req, res).await;
            let mut response = response.header("access-control-allow-origin", &opts.origin);
            if opts.credentials {
                response = response.header("access-control-allow-credentials", "true");
            }
            response
        })
    }
}

/// JSON body parser middleware options
#[derive(Clone)]
pub struct JsonOptions {
    pub limit: usize,
    pub strict: bool,
}

impl Default for JsonOptions {
    fn default() -> Self {
        Self {
            limit: 1024 * 1024, // 1MB
            strict: true,
        }
    }
}

/// JSON body parser middleware
pub fn json(
    options: JsonOptions,
) -> impl Fn(Request, Response, Next) -> Pin<Box<dyn Future<Output = Response> + Send>>
       + Send
       + Sync
       + Clone {
    move |req: Request, res: Response, next: Next| {
        let _options = options.clone();
        Box::pin(async move {
            // Check content type
            if let Some(content_type) = req.content_type() {
                if !content_type.contains("application/json") {
                    // Not JSON, pass through
                    return next(req, res).await;
                }
            }

            // Continue to next middleware/handler
            next(req, res).await
        })
    }
}

/// Compression middleware options
#[derive(Clone)]
pub struct CompressionOptions {
    pub level: u32,
    pub threshold: usize,
}

impl Default for CompressionOptions {
    fn default() -> Self {
        Self {
            level: 6,
            threshold: 1024, // Only compress responses > 1KB
        }
    }
}

/// Security headers middleware (Helmet)
///
/// Adds security headers to protect against common vulnerabilities.
///
/// # Example
///
/// ```rust,ignore
/// use rustyx::middleware::helmet;
///
/// app.use_middleware(helmet());
/// ```
pub fn helmet() -> impl Fn(Request, Response, Next) -> Pin<Box<dyn Future<Output = Response> + Send>>
       + Send
       + Sync
       + Clone {
    move |req: Request, res: Response, next: Next| {
        Box::pin(async move {
            let response = next(req, res).await;
            response
                .header("x-content-type-options", "nosniff")
                .header("x-frame-options", "DENY")
                .header("x-xss-protection", "1; mode=block")
                .header(
                    "strict-transport-security",
                    "max-age=31536000; includeSubDomains",
                )
                .header("content-security-policy", "default-src 'self'")
                .header("x-permitted-cross-domain-policies", "none")
                .header("referrer-policy", "strict-origin-when-cross-origin")
        })
    }
}

/// Request timeout middleware
///
/// # Example
///
/// ```rust,ignore
/// use rustyx::middleware::timeout;
///
/// // 30 second timeout
/// app.use_middleware(timeout(30000));
/// ```
pub fn timeout(
    duration_ms: u64,
) -> impl Fn(Request, Response, Next) -> Pin<Box<dyn Future<Output = Response> + Send>>
       + Send
       + Sync
       + Clone {
    move |req: Request, res: Response, next: Next| {
        Box::pin(async move {
            let timeout = tokio::time::Duration::from_millis(duration_ms);

            match tokio::time::timeout(timeout, next(req, res)).await {
                Ok(response) => response,
                Err(_) => Response::new()
                    .status(408)
                    .json(serde_json::json!({ "error": "Request Timeout" })),
            }
        })
    }
}

/// Request ID middleware - adds unique ID to each request
pub fn request_id(
) -> impl Fn(Request, Response, Next) -> Pin<Box<dyn Future<Output = Response> + Send>>
       + Send
       + Sync
       + Clone {
    move |req: Request, res: Response, next: Next| {
        Box::pin(async move {
            let request_id = uuid::Uuid::new_v4().to_string();
            let response = next(req, res).await;
            response.header("x-request-id", &request_id)
        })
    }
}

/// Response time middleware - adds X-Response-Time header
pub fn response_time(
) -> impl Fn(Request, Response, Next) -> Pin<Box<dyn Future<Output = Response> + Send>>
       + Send
       + Sync
       + Clone {
    move |req: Request, res: Response, next: Next| {
        Box::pin(async move {
            let start = std::time::Instant::now();
            let response = next(req, res).await;
            let duration = start.elapsed();
            response.header("x-response-time", &format!("{}ms", duration.as_millis()))
        })
    }
}
