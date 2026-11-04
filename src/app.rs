//! Main Application Module
//!
//! This module contains the core [`RustyX`] application struct and its implementation,
//! providing an ExpressJS-like interface for building web APIs.
//!
//! # Example
//!
//! ```rust,no_run
//! use rustyx::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let app = RustyX::new();
//!
//!     app.get("/", |_req, res| async move {
//!         res.json(json!({ "message": "Hello!" }))
//!     });
//!
//!     app.listen(3000).await
//! }
//! ```

use crate::error::Result;
use crate::middleware::{MiddlewareStack, Next};
use crate::request::Request;
use crate::response::Response;
use crate::router::Router;

use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{body::Incoming, Method};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

/// Handler function type for route callbacks.
///
/// This is the type signature for route handlers. Handlers receive a [`Request`]
/// and [`Response`], and return a [`Future`] that resolves to a [`Response`].
pub type HandlerFn =
    Arc<dyn Fn(Request, Response) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

/// The main application struct, similar to Express's `app`.
///
/// `RustyX` is the core of your web application. It handles routing,
/// middleware, and server lifecycle.
///
/// # Example
///
/// ```rust,no_run
/// use rustyx::prelude::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let app = RustyX::new();
///
///     // Add middleware
///     app.use_middleware(logger());
///     app.use_middleware(cors("*"));
///
///     // Define routes
///     app.get("/", |_req, res| async move {
///         res.json(json!({ "status": "ok" }))
///     });
///
///     app.get("/users/:id", |req, res| async move {
///         let id = req.param("id").unwrap();
///         res.json(json!({ "user_id": id }))
///     });
///
///     // Start server
///     app.listen(3000).await
/// }
/// ```
pub struct RustyX {
    router: Arc<std::sync::RwLock<Router>>,
    middleware_stack: Arc<std::sync::RwLock<MiddlewareStack>>,
    settings: Arc<std::sync::RwLock<AppSettings>>,
}

/// Application settings configuration.
///
/// Controls various aspects of application behavior.
#[derive(Debug, Clone)]
pub struct AppSettings {
    /// Number of spaces for JSON formatting (None for compact)
    pub json_spaces: Option<usize>,
    /// Whether to trust proxy headers (X-Forwarded-For, etc.)
    pub trust_proxy: bool,
    /// Enable case-sensitive routing
    pub case_sensitive_routing: bool,
    /// Enable strict routing (trailing slashes matter)
    pub strict_routing: bool,
    /// Current environment (development, production, etc.)
    pub env: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            json_spaces: None,
            trust_proxy: false,
            case_sensitive_routing: false,
            strict_routing: false,
            env: std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
        }
    }
}

impl RustyX {
    /// Creates a new RustyX application instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyx::RustyX;
    ///
    /// let app = RustyX::new();
    /// ```
    pub fn new() -> Self {
        Self {
            router: Arc::new(std::sync::RwLock::new(Router::new())),
            middleware_stack: Arc::new(std::sync::RwLock::new(MiddlewareStack::new())),
            settings: Arc::new(std::sync::RwLock::new(AppSettings::default())),
        }
    }

    /// Set an application setting
    pub fn set<T: ToString>(&self, key: &str, value: T) -> &Self {
        if let Ok(mut settings) = self.settings.write() {
            match key {
                "json_spaces" => settings.json_spaces = value.to_string().parse().ok(),
                "trust_proxy" => settings.trust_proxy = value.to_string().parse().unwrap_or(false),
                "case_sensitive_routing" => {
                    settings.case_sensitive_routing = value.to_string().parse().unwrap_or(false)
                }
                "strict_routing" => {
                    settings.strict_routing = value.to_string().parse().unwrap_or(false)
                }
                "env" => settings.env = value.to_string(),
                _ => {}
            }
        }
        self
    }

    /// Add middleware to the application
    pub fn use_middleware<F, Fut>(&self, middleware: F) -> &Self
    where
        F: Fn(Request, Response, Next) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        if let Ok(mut stack) = self.middleware_stack.write() {
            stack.push(Box::new(move |req, res, next| {
                Box::pin(middleware(req, res, next))
            }));
        }
        self
    }

    /// Mount a router at a specific path prefix
    pub fn use_router(&self, path: &str, router: Router) -> &Self {
        if let Ok(mut main_router) = self.router.write() {
            main_router.mount(path, router);
        }
        self
    }

    /// Register a GET route handler
    pub fn get<F, Fut>(&self, path: &str, handler: F) -> &Self
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::GET, path, handler)
    }

    /// Register a POST route handler
    pub fn post<F, Fut>(&self, path: &str, handler: F) -> &Self
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::POST, path, handler)
    }

    /// Register a PUT route handler
    pub fn put<F, Fut>(&self, path: &str, handler: F) -> &Self
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::PUT, path, handler)
    }

    /// Register a DELETE route handler
    pub fn delete<F, Fut>(&self, path: &str, handler: F) -> &Self
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::DELETE, path, handler)
    }

    /// Register a PATCH route handler
    pub fn patch<F, Fut>(&self, path: &str, handler: F) -> &Self
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::PATCH, path, handler)
    }

    /// Internal method to register a route
    fn route<F, Fut>(&self, method: Method, path: &str, handler: F) -> &Self
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        if let Ok(mut router) = self.router.write() {
            router.add_route(
                method,
                path,
                Arc::new(move |req, res| Box::pin(handler(req, res))),
            );
        }
        self
    }

    /// Start the HTTP server and listen on the specified port
    pub async fn listen(self, port: u16) -> Result<()> {
        self.listen_with_callback(port, || {
            info!("🚀 RustyX server running at http://localhost:{}", port);
        })
        .await
    }

    /// Start the HTTP server with a callback that's called once the server starts
    pub async fn listen_with_callback<F>(self, port: u16, callback: F) -> Result<()>
    where
        F: FnOnce(),
    {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(addr).await?;

        callback();

        let app = Arc::new(self);

        loop {
            let (stream, remote_addr) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let app = Arc::clone(&app);

            tokio::spawn(async move {
                let service = service_fn(move |req: hyper::Request<Incoming>| {
                    let app = Arc::clone(&app);
                    async move {
                        let response = app.handle_request(req, remote_addr).await;
                        Ok::<_, Infallible>(response)
                    }
                });

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    error!("Error serving connection: {:?}", err);
                }
            });
        }
    }

    /// Handle an incoming HTTP request
    async fn handle_request(
        &self,
        req: hyper::Request<Incoming>,
        remote_addr: SocketAddr,
    ) -> hyper::Response<Full<Bytes>> {
        // Convert hyper request to our Request type
        let request = match Request::from_hyper(req, remote_addr).await {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to parse request: {:?}", e);
                return Response::new()
                    .status(400)
                    .json(serde_json::json!({ "error": "Bad Request" }))
                    .into_hyper();
            }
        };

        let response = Response::new();

        // Find and execute the route handler
        let handler_and_params = {
            let router = self.router.read().unwrap();
            router
                .find_route(request.method(), request.path())
                .map(|(h, p)| (Arc::clone(h), p))
        };

        if let Some((handler, params)) = handler_and_params {
            let mut request = request;
            request.set_params(params);

            let response = handler(request, response).await;
            response.into_hyper()
        } else {
            Response::new()
                .status(404)
                .json(serde_json::json!({ "error": "Not Found" }))
                .into_hyper()
        }
    }
}

impl Default for RustyX {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RustyX {
    fn clone(&self) -> Self {
        Self {
            router: Arc::clone(&self.router),
            middleware_stack: Arc::clone(&self.middleware_stack),
            settings: Arc::clone(&self.settings),
        }
    }
}
