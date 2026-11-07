//! # Nitrox
//!
//! A fast, minimalist web framework for Rust inspired by ExpressJS.
//!
//! Nitrox provides an ExpressJS-like interface for building high-performance web APIs in Rust.
//!
//! ## Features
//!
//! - **ExpressJS-like API** - Familiar interface for JavaScript and TypeScript developers
//! - **High Performance** - Built on Hyper and Tokio for high-throughput, low-latency execution
//! - **Zero-Clone State Management** - Type-safe shared application state via `req.state`
//! - **Middleware Support** - Logger, CORS, Rate Limiting, Helmet, Timeout
//! - **File Upload** - Multer-like multipart file upload with validation
//! - **WebSocket Support** - Real-time bidirectional communication
//! - **Static Files** - Serve static assets with automatic MIME type detection
//! - **Type-Safe** - Leverage Rust's type system for safer, robust code
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use nitrox::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let app = Nitrox::new();
//!
//!     app.get("/", |_req, res| async move {
//!         res.json(json!({ "message": "Hello, World!" }))
//!     });
//!
//!     app.listen(3000).await
//! }
//! ```
//!
//! ## Routing
//!
//! Nitrox supports all common HTTP methods with ExpressJS-style routing:
//!
//! ```rust,no_run
//! use nitrox::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let app = Nitrox::new();
//!
//!     // GET request
//!     app.get("/users", |_req, res| async move {
//!         res.json(json!({ "users": [] }))
//!     });
//!
//!     // POST request
//!     app.post("/users", |req, res| async move {
//!         // Parse JSON body
//!         let data: serde_json::Value = req.json().unwrap_or_default();
//!         res.status(201).json(data)
//!     });
//!
//!     // URL parameters
//!     app.get("/users/:id", |req, res| async move {
//!         let id = req.param("id").unwrap();
//!         res.json(json!({ "user_id": id }))
//!     });
//!
//!     app.listen(3000).await
//! }
//! ```
//!
//! ## Application State Management
//!
//! Nitrox supports global, type-safe shared state using [`Nitrox::with_state`].
//! State is automatically wrapped in an [`Arc`] and injected into every [`Request`],
//! accessible via `req.state` without manual closure cloning:
//!
//! ```rust,no_run
//! use nitrox::prelude::*;
//!
//! struct AppState {
//!     environment: String,
//!     api_version: &'static str,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let state = AppState {
//!         environment: "production".to_string(),
//!         api_version: "v1",
//!     };
//!
//!     let app = Nitrox::with_state(state);
//!
//!     app.get("/info", |req, res| async move {
//!         res.json(json!({
//!             "env": req.state.environment,
//!             "version": req.state.api_version
//!         }))
//!     });
//!
//!     app.listen(3000).await
//! }
//! ```
//!
//! ### State Requirements and Limitations
//!
//! **Trait Bounds:**
//! Any type passed into `Nitrox::with_state(state)` must implement `Send + Sync + 'static`:
//! - `Send`: Safe to transfer across multi-threaded asynchronous tasks.
//! - `Sync`: Safe for concurrent read references across multiple worker threads.
//! - `'static`: Must own its data or hold static references (no short-lived temporary stack borrows).
//!
//! **What CAN Be Passed:**
//! - Connection pools: `sqlx::PgPool`, `sqlx::MySqlPool`, `deadpool_redis::Pool`, `bb8::Pool`
//! - HTTP clients: `reqwest::Client` singletons
//! - Thread-safe synchronized structures: `dashmap::DashMap`, `parking_lot::RwLock<HashMap<K, V>>`, `moka::future::Cache`
//! - Background channels: `tokio::sync::mpsc::Sender<T>`, `tokio::sync::broadcast::Sender<T>`
//! - Atomic indicators: `std::sync::atomic::AtomicU64`, `AtomicBool`
//! - Read-mostly configurations: `arc_swap::ArcSwap<Config>`, immutable data structs
//!
//! **What CANNOT Be Passed (and Anti-Patterns to Avoid):**
//! - **Non-`Send` / Non-`Sync` types:** Single-threaded primitives like `Rc<T>`, `RefCell<T>`,
//!   `std::cell::Cell<T>`, and raw pointers (`*const T`, `*mut T`) cannot cross thread boundaries.
//!   Use `Arc<T>` and `parking_lot::RwLock<T>` or `tokio::sync::RwLock<T>` instead.
//! - **Non-`'static` lifetimes:** Structs containing temporary borrows (`&'a str` or `&'a T`)
//!   are rejected at compile time because the server lifecycle is indefinite. Store owned `String` or `Arc<T>`.
//! - **`std::sync::Mutex` held across `.await`:** Standard library mutex guards are not `Send` and will
//!   block the Tokio worker thread runtime. Use `tokio::sync::Mutex` if a lock must be held across
//!   asynchronous `.await` operations, or `parking_lot::RwLock` for synchronous critical sections.
//! - **Un-pooled single connections:** Raw single-threaded drivers (e.g. un-pooled `rusqlite::Connection`)
//!   fail the `Sync` requirement. Always wrap single connections in a connection pool manager.
//!
//! ## Middleware
//!
//! Add middleware for cross-cutting concerns:
//!
//! ```rust,no_run
//! use nitrox::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let app = Nitrox::new();
//!
//!     // Built-in middleware
//!     app.use_middleware(logger());          // Request logging
//!     app.use_middleware(cors("*"));         // CORS headers
//!     app.use_middleware(helmet());          // Security headers
//!     app.use_middleware(timeout(30000));    // 30s timeout
//!
//!     // Rate limiting
//!     let rate_config = RateLimiterConfig::new(100, 60);
//!     app.use_middleware(rate_limiter(rate_config));
//!
//!     app.get("/", |_req, res| async move {
//!         res.json(json!({ "status": "ok" }))
//!     });
//!
//!     app.listen(3000).await
//! }
//! ```
//!
//! ## Request Object
//!
//! The [`Request`] object provides access to request data:
//!
//! - `req.method()` - HTTP method
//! - `req.path()` - Request path
//! - `req.param("name")` - URL parameters
//! - `req.query_param("key")` - Query parameters
//! - `req.json::<T>()` - Parse JSON body
//! - `req.header("name")` - Get header value
//! - `req.bearer_token()` - Extract Bearer token
//! - `req.ip()` - Client IP address
//!
//! ## Response Object
//!
//! The [`Response`] object provides methods for sending responses:
//!
//! - `res.json(data)` - Send JSON response
//! - `res.send("text")` - Send text response
//! - `res.html("<html>")` - Send HTML response
//! - `res.status(201)` - Set status code
//! - `res.redirect("/path")` - Send redirect
//! - `res.header("name", "value")` - Set header
//!
//! ## File Upload
//!
//! Handle file uploads similar to Express Multer:
//!
//! ```rust,no_run
//! use nitrox::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let app = Nitrox::new();
//!
//!     // Create uploader configuration
//!     let uploader = Uploader::new(
//!         UploadConfig::new()
//!             .destination("./uploads")
//!             .max_file_size_mb(5)
//!             .allowed_extensions(vec!["png", "jpg", "jpeg", "pdf"])
//!     );
//!
//!     app.post("/upload", move |req, res| {
//!         let uploader = uploader.clone();
//!         async move {
//!             // Parse multipart form data
//!             let content_type = req.content_type().unwrap_or_default();
//!             let boundary = parse_boundary(&content_type).unwrap();
//!             let fields = parse_multipart(req.body(), &boundary).unwrap();
//!             
//!             for field in fields {
//!                 if let Some(filename) = field.filename {
//!                     let result = uploader.upload_single(
//!                         &field.name,
//!                         field.data,
//!                         &filename,
//!                         &field.content_type.unwrap_or_default()
//!                     ).await;
//!                     
//!                     match result {
//!                         Ok(file) => return res.json(json!({
//!                             "filename": file.filename,
//!                             "size": file.size
//!                         })),
//!                         Err(e) => return res.bad_request(&e.to_string())
//!                     }
//!                 }
//!             }
//!             res.bad_request("No file provided")
//!         }
//!     });
//!
//!     app.listen(3000).await
//! }
//! ```
//!
//! ### Upload Configuration Options
//!
//! | Method | Description |
//! |--------|-------------|
//! | `.destination("./uploads")` | Set upload directory |
//! | `.max_file_size_mb(10)` | Max file size in MB |
//! | `.max_files(5)` | Max files per request |
//! | `.images_only()` | Only allow image files |
//! | `.documents_only()` | Only allow document files |
//! | `.allowed_extensions(vec!["png", "pdf"])` | Custom allowed extensions |
//! | `.allowed_types(vec!["image/png"])` | Custom allowed MIME types |
//! | `.keep_original_name()` | Keep original filename |
//! | `.use_uuid()` | Use UUID for filename |
//!
//! ### Supported File Types
//!
//! **Images:** PNG, JPG, JPEG, GIF, WebP, SVG, ICO, BMP, TIFF
//!
//! **Documents:** PDF, DOC, DOCX, XLS, XLSX, PPT, PPTX, TXT, CSV
//!
//! **Media:** MP3, WAV, OGG, MP4, WebM, AVI, MOV
//!
//! **Archives:** ZIP, RAR, 7Z, TAR, GZ
//!
//! ## Modules
//!
//! - [`app`] - Main application struct
//! - [`router`] - Routing functionality
//! - [`request`] - Request handling
//! - [`response`] - Response building
//! - [`middleware`] - Middleware functions
//! - [`upload`] - File upload handling
//! - [`websocket`] - WebSocket support
//! - [`static_files`] - Static file serving
//!
//! ## Database Support
//!
//! Nitrox is completely database agnostic. Manage connection pools (`sqlx`, `diesel`,
//! `sea-orm`, `mongodb`, or `redis`) inside your custom application state struct and
//! pass it to `Nitrox::with_state(state)`. Handlers access connection pools directly via `req.state.db`.

#![doc(html_root_url = "https://docs.rs/nitrox/0.0.1")]
#![allow(missing_docs)] // TODO: Add docs for all public items before 1.0
#![warn(rustdoc::missing_crate_level_docs)]

pub mod app;
pub mod controllers;
pub mod error;
pub mod middleware;
pub mod models;
pub mod request;
pub mod response;
pub mod router;
pub mod routes;
pub mod static_files;
pub mod upload;
pub mod utils;
pub mod websocket;

// Re-exports for convenience
pub use app::{Nitrox, RustyX};
pub use error::{Error, Result};
pub use middleware::{IntoMiddlewareChain, Middleware, MiddlewareFn, Next};
pub use request::Request;
pub use response::Response;
pub use router::{RouteBuilder, Router};
pub use static_files::{static_handler, StaticConfig};
pub use upload::{UploadConfig, UploadedFile, Uploader};
pub use websocket::{WsMessage, WsRoom, WsServer};

#[cfg(target_os = "linux")]
pub use tikv_jemallocator;

/// Configures `jemalloc` as the global memory allocator on Linux for peak multi-threaded throughput.
///
/// On non-Linux platforms (Windows, macOS), this macro safely expands to nothing,
/// allowing the application to compile and run with the native platform allocator.
///
/// # Example
///
/// ```rust,no_run
/// use nitrox::prelude::*;
///
/// nitrox::enable_jemalloc!();
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let app = Nitrox::new();
///     app.listen(3000).await
/// }
/// ```
#[macro_export]
macro_rules! enable_jemalloc {
    () => {
        #[cfg(target_os = "linux")]
        #[global_allocator]
        static __NITROX_GLOBAL_ALLOCATOR: $crate::tikv_jemallocator::Jemalloc =
            $crate::tikv_jemallocator::Jemalloc;
    };
}

/// Prelude module for convenient imports.
///
/// Import everything you need with a single line:
///
/// ```rust
/// use nitrox::prelude::*;
/// ```
///
/// This includes:
/// - [`Nitrox`] / [`RustyX`] - Main application struct
/// - [`Request`] and [`Response`] - HTTP handling
/// - [`Router`] - Route grouping
/// - All middleware functions
/// - Serde traits and macros
/// - Tracing macros
pub mod prelude {
    pub use crate::app::{Nitrox, RustyX};
    pub use crate::controllers::Controller;
    pub use crate::error::{Error, Result};
    pub use crate::middleware::{
        cors, cors_with_options, helmet, json, logger, rate_limiter, request_id, response_time,
        simple_rate_limit, timeout, CorsOptions, IntoMiddlewareChain, JsonOptions, Middleware,
        MiddlewareFn, Next, RateLimiterConfig,
    };
    pub use crate::models::Model;
    pub use crate::request::Request;
    pub use crate::response::{CookieOptions, Response};
    pub use crate::router::{RouteBuilder, Router};
    pub use crate::static_files::{static_handler, StaticConfig};
    pub use crate::upload::{
        parse_boundary, parse_multipart, FileNaming, MultipartField, StorageType, UploadConfig,
        UploadError, UploadedFile, Uploader,
    };
    pub use crate::websocket::{WsMessage, WsRoom, WsServer};
    pub use crate::enable_jemalloc;
    pub use async_trait::async_trait;
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{json, Value};
    pub use tracing::{debug, error, info, trace, warn};
}

/// Version of the Nitrox library.
///
/// # Example
///
/// ```rust
/// println!("Nitrox version: {}", nitrox::VERSION);
/// ```
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Name of the library.
pub const NAME: &str = "Nitrox";

/// GitHub repository URL.
pub const REPOSITORY: &str = "https://github.com/KTBsomen/nitrox";

/// Documentation URL.
pub const DOCS_URL: &str = "https://docs.rs/nitrox";
