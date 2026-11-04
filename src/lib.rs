//! # RustyX
//!
//! A fast, minimalist web framework for Rust inspired by ExpressJS.
//!
//! RustyX provides an ExpressJS-like interface for building web APIs in Rust,
//! with built-in ORM support for MongoDB, MySQL, SQLite, and PostgreSQL.
//!
//! ## Features
//!
//! - 🎯 **ExpressJS-like API** - Familiar interface for JavaScript developers
//! - ⚡ **Blazingly Fast** - Built on Hyper and Tokio for maximum performance
//! - 🔌 **Middleware Support** - Logger, CORS, Rate Limiting, Helmet, Timeout
//! - 📤 **File Upload** - Multer-like file upload with validation
//! - 🗄️ **Multi-Database ORM** - MongoDB, MySQL, PostgreSQL, SQLite support
//! - 🌐 **WebSocket Support** - Real-time bidirectional communication
//! - 📁 **Static Files** - Serve static assets with MIME type detection
//! - 🔒 **Type-Safe** - Leverage Rust's type system for safer code
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rustyx::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let app = RustyX::new();
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
//! RustyX supports all common HTTP methods with ExpressJS-style routing:
//!
//! ```rust,no_run
//! use rustyx::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let app = RustyX::new();
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
//! ## Middleware
//!
//! Add middleware for cross-cutting concerns:
//!
//! ```rust,no_run
//! use rustyx::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let app = RustyX::new();
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
//! use rustyx::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let app = RustyX::new();
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
//! ## Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `default` | SQLite support enabled |
//! | `mysql` | MySQL database support |
//! | `postgres` | PostgreSQL database support |
//! | `sqlite` | SQLite database support |
//! | `mongodb` | MongoDB database support |
//! | `full` | All database drivers enabled |
//!
//! ## Modules
//!
//! - [`app`] - Main application struct
//! - [`router`] - Routing functionality
//! - [`request`] - Request handling
//! - [`response`] - Response building
//! - [`middleware`] - Middleware functions
//! - [`upload`] - File upload handling
//! - [`db`] - Database integration
//! - [`websocket`] - WebSocket support
//! - [`static_files`] - Static file serving

#![doc(html_root_url = "https://docs.rs/rustyx/0.2.0")]
#![allow(missing_docs)] // TODO: Add docs for all public items before 1.0
#![warn(rustdoc::missing_crate_level_docs)]

pub mod app;
pub mod controllers;
pub mod db;
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
pub use app::RustyX;
pub use error::{Error, Result};
pub use middleware::{Middleware, MiddlewareFn, Next};
pub use request::Request;
pub use response::Response;
pub use router::Router;
pub use static_files::{static_handler, StaticConfig};
pub use upload::{UploadConfig, UploadedFile, Uploader};
pub use websocket::{WsMessage, WsRoom, WsServer};

/// Prelude module for convenient imports.
///
/// Import everything you need with a single line:
///
/// ```rust
/// use rustyx::prelude::*;
/// ```
///
/// This includes:
/// - [`RustyX`] - Main application struct
/// - [`Request`] and [`Response`] - HTTP handling
/// - [`Router`] - Route grouping
/// - All middleware functions
/// - Serde traits and macros
/// - Tracing macros
pub mod prelude {
    pub use crate::app::RustyX;
    pub use crate::controllers::Controller;
    pub use crate::db::prelude::*;
    pub use crate::error::{Error, Result};
    pub use crate::middleware::{
        cors, cors_with_options, helmet, json, logger, rate_limiter, request_id, response_time,
        simple_rate_limit, timeout, CorsOptions, JsonOptions, Middleware, MiddlewareFn, Next,
        RateLimiterConfig,
    };
    pub use crate::models::Model;
    pub use crate::request::Request;
    pub use crate::response::{CookieOptions, Response};
    pub use crate::router::Router;
    pub use crate::static_files::{static_handler, StaticConfig};
    pub use crate::upload::{
        parse_boundary, parse_multipart, FileNaming, MultipartField, StorageType, UploadConfig,
        UploadError, UploadedFile, Uploader,
    };
    pub use crate::websocket::{WsMessage, WsRoom, WsServer};
    pub use async_trait::async_trait;
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{json, Value};
    pub use tracing::{debug, error, info, trace, warn};
}

/// Version of the RustyX library.
///
/// # Example
///
/// ```rust
/// println!("RustyX version: {}", rustyx::VERSION);
/// ```
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Name of the library.
pub const NAME: &str = "RustyX";

/// GitHub repository URL.
pub const REPOSITORY: &str = "https://github.com/Mohammad007/rustyx";

/// Documentation URL.
pub const DOCS_URL: &str = "https://docs.rs/rustyx";
