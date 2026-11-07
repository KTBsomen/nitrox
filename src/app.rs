//! Main Application Module
//!
//! This module contains the core [`Nitrox`] application struct and its implementation,
//! providing an ExpressJS-like interface for building web APIs.
//!
//! # Example
//!
//! ```rust,no_run
//! use nitrox::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let app = Nitrox::new();
//!
//!     app.get("/", |_req, res| async move {
//!         res.json(json!({ "message": "Hello!" }))
//!     });
//!
//!     app.listen(3000).await
//! }
//! ```

use crate::error::Result;
use crate::middleware::{IntoMiddlewareChain, MiddlewareFn, MiddlewareStack, Next};
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
pub type HandlerFn<S = ()> =
    Arc<dyn Fn(Request<S>, Response) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

/// Immutable server state used during serving to achieve lock-free routing and middleware dispatch.
pub struct ServerState<S = ()> {
    pub router: Router<S>,
    pub middlewares: Arc<Vec<MiddlewareFn<S>>>,
    #[allow(dead_code)]
    pub settings: AppSettings,
    pub state: Arc<S>,
}

impl<S: Send + Sync + 'static> ServerState<S> {
    /// Handle an incoming HTTP request without locking
    pub async fn handle_request(
        &self,
        req: hyper::Request<Incoming>,
        remote_addr: SocketAddr,
    ) -> hyper::Response<Full<Bytes>> {
        // Convert hyper request to our Request type with application state
        let request = match Request::from_hyper_with_state(req, remote_addr, Arc::clone(&self.state)).await {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to parse request: {:?}", e);
                return Response::new()
                    .status(400)
                    .json(serde_json::json!({ "error": e.to_string() }))
                    .into_hyper();
            }
        };

        let response = Response::new();

        // Lock-free route lookup directly on immutable router without Arc cloning on fast path
        if let Some((handler_ref, params)) = self.router.find_route(request.method(), request.path()) {
            let mut request = request;
            if !params.is_empty() {
                request.set_params(params);
            }

            if self.middlewares.is_empty() {
                // Zero-overhead fast path: invoke route handler directly without Arc clone
                let response = handler_ref(request, response).await;
                response.into_hyper()
            } else {
                // Execute middleware pipeline: clone Arc only when branching into middlewares
                let handler = Arc::clone(handler_ref);
                let mws = Arc::clone(&self.middlewares);
                let response = Self::run_middleware_chain(mws, 0, request, response, handler).await;
                response.into_hyper()
            }
        } else {
            Response::new()
                .status(404)
                .json(serde_json::json!({ "error": "Not Found" }))
                .into_hyper()
        }
    }

    /// Recursively compose and execute middleware in registered order
    fn run_middleware_chain(
        middlewares: Arc<Vec<MiddlewareFn<S>>>,
        index: usize,
        req: Request<S>,
        res: Response,
        terminal_handler: HandlerFn<S>,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        if index < middlewares.len() {
            let mw = Arc::clone(&middlewares[index]);
            let next_mws = Arc::clone(&middlewares);
            let next_handler = Arc::clone(&terminal_handler);
            let next: Next<S> = Arc::new(move |req, res| {
                Self::run_middleware_chain(
                    Arc::clone(&next_mws),
                    index + 1,
                    req,
                    res,
                    Arc::clone(&next_handler),
                )
            });
            mw(req, res, next)
        } else {
            terminal_handler(req, res)
        }
    }
}

/// The main application struct, similar to Express's `app`.
///
/// `Nitrox` is the core of your web application. It handles routing,
/// middleware, and server lifecycle.
///
/// # Example
///
/// ```rust,no_run
/// use nitrox::prelude::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let app = Nitrox::new();
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
pub struct Nitrox<S = ()> {
    router: Arc<std::sync::RwLock<Router<S>>>,
    middleware_stack: Arc<std::sync::RwLock<MiddlewareStack<S>>>,
    settings: Arc<std::sync::RwLock<AppSettings>>,
    state: Arc<S>,
}

/// Backward compatibility alias for [`Nitrox`].
pub type RustyX<S = ()> = Nitrox<S>;

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

impl Nitrox<()> {
    /// Creates a new Nitrox application instance with default empty unit state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nitrox::Nitrox;
    ///
    /// let app = Nitrox::new();
    /// ```
    pub fn new() -> Self {
        Self::with_state(())
    }
}

impl<S: Send + Sync + 'static> Nitrox<S> {
    /// Creates a new Nitrox application instance with shared application state.
    ///
    /// The state type `S` is automatically wrapped in an [`Arc`] and injected into
    /// every [`Request`], accessible directly via `req.state`.
    ///
    /// # Trait Bounds and Requirements
    ///
    /// The application state `S` must satisfy:
    /// - [`Send`] - Safe to transfer across thread boundaries.
    /// - [`Sync`] - Safe for concurrent references across multiple threads.
    /// - `'static` - Outlives temporary function or stack scopes.
    ///
    /// # Permitted State Types
    ///
    /// The following types are safe and recommended for use in application state:
    /// - Connection pools (`sqlx::PgPool`, `sqlx::MySqlPool`, `deadpool_redis::Pool`, `bb8::Pool`)
    /// - HTTP clients (`reqwest::Client`)
    /// - Thread-safe synchronized caches (`dashmap::DashMap`, `parking_lot::RwLock<HashMap<K, V>>`, `moka::future::Cache`)
    /// - Tokio channels (`tokio::sync::mpsc::Sender`, `tokio::sync::broadcast::Sender`)
    /// - Atomic primitives (`std::sync::atomic::AtomicU64`, `AtomicBool`)
    /// - Read-mostly synchronized configuration (`arc_swap::ArcSwap<Config>`)
    ///
    /// # Limitations and Disallowed Types
    ///
    /// - **Non-Send/Non-Sync types:** Types such as `Rc<T>`, `RefCell<T>`, `std::cell::Cell<T>`,
    ///   or raw pointers cannot be used because requests are processed concurrently across Tokio worker threads.
    ///   Use `Arc<T>` and `parking_lot::RwLock<T>` or `tokio::sync::RwLock<T>` instead.
    /// - **Non-'static references:** Borrowed references with temporary lifetimes (`&'a T`) cannot be stored
    ///   in state; state must own its data or hold `'static` references.
    /// - **Blocking Mutex across await:** Avoid holding `std::sync::MutexGuard` across an `.await` boundary,
    ///   as it is not `Send` and can cause worker thread pool starvation. Use `tokio::sync::Mutex` if a lock
    ///   must span across asynchronous operations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nitrox::prelude::*;
    ///
    /// struct AppState {
    ///     env: String,
    /// }
    ///
    /// let app = Nitrox::with_state(AppState { env: "production".to_string() });
    /// ```
    pub fn with_state(state: S) -> Self {
        Self {
            router: Arc::new(std::sync::RwLock::new(Router::with_state())),
            middleware_stack: Arc::new(std::sync::RwLock::new(MiddlewareStack::new())),
            settings: Arc::new(std::sync::RwLock::new(AppSettings::default())),
            state: Arc::new(state),
        }
    }

    /// Access reference to application state
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Clone the Arc pointing to application state
    pub fn state_arc(&self) -> Arc<S> {
        Arc::clone(&self.state)
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
        F: Fn(Request<S>, Response, Next<S>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        if let Ok(mut stack) = self.middleware_stack.write() {
            stack.push(Arc::new(move |req, res, next| {
                Box::pin(middleware(req, res, next))
            }));
        }
        self
    }

    /// Mount a router at a specific path prefix
    pub fn use_router(&self, path: &str, router: Router<S>) -> &Self {
        if let Ok(mut main_router) = self.router.write() {
            main_router.mount(path, router);
        }
        self
    }

    /// Alias for `use_router`. Mounts a sub-router at a specific path prefix.
    pub fn mount(&self, path: &str, router: Router<S>) -> &Self {
        self.use_router(path, router)
    }

    /// Register a GET route handler
    pub fn get<F, Fut>(&self, path: &str, handler: F) -> &Self
    where
        F: Fn(Request<S>, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::GET, path, handler)
    }

    /// Register a POST route handler
    pub fn post<F, Fut>(&self, path: &str, handler: F) -> &Self
    where
        F: Fn(Request<S>, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::POST, path, handler)
    }

    /// Register a PUT route handler
    pub fn put<F, Fut>(&self, path: &str, handler: F) -> &Self
    where
        F: Fn(Request<S>, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::PUT, path, handler)
    }

    /// Register a DELETE route handler
    pub fn delete<F, Fut>(&self, path: &str, handler: F) -> &Self
    where
        F: Fn(Request<S>, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::DELETE, path, handler)
    }

    /// Register a PATCH route handler
    pub fn patch<F, Fut>(&self, path: &str, handler: F) -> &Self
    where
        F: Fn(Request<S>, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::PATCH, path, handler)
    }

    /// Register a GET route with route-level middleware(s).
    pub fn get_with<M, H, Fut>(&self, path: &str, middlewares: M, handler: H) -> &Self
    where
        M: IntoMiddlewareChain<S>,
        H: Fn(Request<S>, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        if let Ok(mut router) = self.router.write() {
            router.get_with(path, middlewares, handler);
        }
        self
    }

    /// Register a POST route with route-level middleware(s).
    pub fn post_with<M, H, Fut>(&self, path: &str, middlewares: M, handler: H) -> &Self
    where
        M: IntoMiddlewareChain<S>,
        H: Fn(Request<S>, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        if let Ok(mut router) = self.router.write() {
            router.post_with(path, middlewares, handler);
        }
        self
    }

    /// Register a PUT route with route-level middleware(s).
    pub fn put_with<M, H, Fut>(&self, path: &str, middlewares: M, handler: H) -> &Self
    where
        M: IntoMiddlewareChain<S>,
        H: Fn(Request<S>, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        if let Ok(mut router) = self.router.write() {
            router.put_with(path, middlewares, handler);
        }
        self
    }

    /// Register a DELETE route with route-level middleware(s).
    pub fn delete_with<M, H, Fut>(&self, path: &str, middlewares: M, handler: H) -> &Self
    where
        M: IntoMiddlewareChain<S>,
        H: Fn(Request<S>, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        if let Ok(mut router) = self.router.write() {
            router.delete_with(path, middlewares, handler);
        }
        self
    }

    /// Register a PATCH route with route-level middleware(s).
    pub fn patch_with<M, H, Fut>(&self, path: &str, middlewares: M, handler: H) -> &Self
    where
        M: IntoMiddlewareChain<S>,
        H: Fn(Request<S>, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        if let Ok(mut router) = self.router.write() {
            router.patch_with(path, middlewares, handler);
        }
        self
    }

    /// Internal method to register a route
    fn route<F, Fut>(&self, method: Method, path: &str, handler: F) -> &Self
    where
        F: Fn(Request<S>, Response) -> Fut + Send + Sync + 'static,
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
            info!("Nitrox server running at http://localhost:{}", port);
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

        // Freeze application state into an immutable ServerState (Zero runtime locks)
        let server_state = Arc::new(ServerState {
            router: self.router.read().unwrap().clone(),
            middlewares: Arc::new(self.middleware_stack.read().unwrap().get_middlewares()),
            settings: self.settings.read().unwrap().clone(),
            state: Arc::clone(&self.state),
        });

        loop {
            let (stream, remote_addr) = listener.accept().await?;
            // Disable Nagle's algorithm for sub-millisecond response delivery
            let _ = stream.set_nodelay(true);

            let io = TokioIo::new(stream);
            let state = Arc::clone(&server_state);

            tokio::spawn(async move {
                let service = service_fn(move |req: hyper::Request<Incoming>| {
                    let state = Arc::clone(&state);
                    async move {
                        let response = state.handle_request(req, remote_addr).await;
                        Ok::<_, Infallible>(response)
                    }
                });

                let mut builder = http1::Builder::new();
                builder.pipeline_flush(true);

                if let Err(err) = builder.serve_connection(io, service).await {
                    error!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

impl Default for Nitrox<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Clone for Nitrox<S> {
    fn clone(&self) -> Self {
        Self {
            router: Arc::clone(&self.router),
            middleware_stack: Arc::clone(&self.middleware_stack),
            settings: Arc::clone(&self.settings),
            state: Arc::clone(&self.state),
        }
    }
}
