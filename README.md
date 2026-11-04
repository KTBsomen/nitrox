# 🚀 Nitrox

<div align="center">

![Nitrox Logo](https://img.shields.io/badge/Nitrox-Fast%20Web%20Framework-orange?style=for-the-badge&logo=rust)

[![Crates.io](https://img.shields.io/crates/v/nitrox.svg?style=flat-square)](https://crates.io/crates/nitrox)
[![Documentation](https://docs.rs/nitrox/badge.svg)](https://docs.rs/nitrox)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/github/actions/workflow/status/KTBsomen/nitrox/ci.yml?branch=main&style=flat-square)](https://github.com/KTBsomen/nitrox/actions)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-blue.svg?style=flat-square)](https://www.rust-lang.org)

**A fast, minimalist web framework for Rust inspired by ExpressJS**

[Getting Started](#-getting-started) •
[Installation](#-installation) •
[Documentation](#-documentation) •
[Examples](#-examples) •
[Deployment](#-deployment) •
[Contributing](#-contributing)

</div>

---

## 📖 Table of Contents

- [Features](#-features)
- [Installation](#-installation)
- [Getting Started](#-getting-started)
- [Core Concepts](#-core-concepts)
  - [Application](#application)
  - [Routing](#routing)
  - [Request](#request)
  - [Response](#response)
  - [Middleware](#middleware)
- [Advanced Features](#-advanced-features)
  - [File Upload](#file-upload)
  - [Rate Limiting](#rate-limiting)
  - [WebSocket Support](#websocket-support)
  - [Static File Serving](#static-file-serving)
  - [Database Integration](#database-integration)
- [Deployment](#-deployment)
- [API Reference](#-api-reference)
- [Examples](#-examples)
- [Contributing](#-contributing)
- [License](#-license)

---

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🎯 **ExpressJS-like API** | Familiar interface for JavaScript/TypeScript developers |
| ⚡ **Blazingly Fast** | Built directly on Hyper and Tokio for high-throughput performance |
| 🔌 **Middleware Support** | Logger, CORS, Rate Limiting, Helmet, Timeout |
| 🗄️ **Database Agnostic** | Seamlessly plug in SQLx, Diesel, SeaORM, MongoDB, or Redis |
| 🌐 **WebSocket Support** | Real-time bidirectional communication |
| 📁 **Static Files** | Serve static assets with automatic MIME type detection |
| 📤 **File Upload** | Multer-like multipart file upload with validation |
| 🔒 **Type-Safe** | Leverage Rust's type system for safer, robust code |
| 📝 **JSON-First** | Designed from the ground up for building REST APIs |
| 🛡️ **Security** | Built-in Helmet middleware for essential security headers |
| ⏱️ **Rate Limiting** | Protect endpoints and APIs from abuse |

---

## 📦 Installation

### Prerequisites

- **Rust 1.75+** - [Install Rust](https://rustup.rs/)
- **Cargo** - Comes with Rust

### Add to Cargo.toml

```toml
[dependencies]
nitrox = "0.0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## 🚀 Getting Started

Create a minimal REST API in under 20 lines of code:

```rust
use nitrox::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Nitrox::new();

    // Middleware
    app.use_middleware(logger());
    app.use_middleware(cors("*"));

    // Routes
    app.get("/", |_req, res| async move {
        res.json(json!({ "message": "Welcome to Nitrox!" }))
    });

    app.get("/users/:id", |req, res| async move {
        let id = req.param("id").unwrap_or_default();
        res.json(json!({ "user_id": id }))
    });

    app.listen(3000).await
}
```

Run your application:

```bash
cargo run
```

---

## 🧩 Core Concepts

### Application

The `Nitrox` struct is the main entry point for configuring routes, middleware, and starting the HTTP server:

```rust
use nitrox::prelude::*;

let app = Nitrox::new();

// Application settings
app.set("env", "production");
app.set("trust_proxy", true);

// Start server
app.listen(3000).await?;
```

### Routing

Nitrox supports all standard HTTP methods with parameter extraction:

```rust
// HTTP Methods
app.get("/api/items", get_items);
app.post("/api/items", create_item);
app.put("/api/items/:id", update_item);
app.delete("/api/items/:id", delete_item);
app.patch("/api/items/:id", patch_item);

// Modular Sub-Routers
let mut api_router = Router::new();
api_router.get("/status", |_req, res| async move {
    res.json(json!({ "status": "healthy" }))
});

app.use_router("/api/v1", api_router);
```

### Request

Access parameters, headers, query strings, and request bodies easily:

```rust
app.post("/api/users/:id", |req, res| async move {
    // Route parameter
    let id = req.param("id");

    // Query parameter (?page=2)
    let page = req.query_param("page");

    // Header value
    let auth = req.header("Authorization");

    // Bearer token
    let token = req.bearer_token();

    // Parse JSON body
    let body: UserDto = req.json()?;

    res.json(json!({ "success": true }))
});
```

### Response

Fluent, chainable response builder:

```rust
app.get("/example", |_req, res| async move {
    res.status(200)
        .header("X-Powered-By", "Nitrox")
        .cookie("session_id", "abc123xyz", CookieOptions::new().http_only(true).secure(true))
        .json(json!({ "data": "value" }))
});
```

### Middleware

Express-style middleware pipeline:

```rust
app.use_middleware(logger());
app.use_middleware(cors("*"));
app.use_middleware(helmet());
app.use_middleware(timeout(30000)); // 30 seconds

// Custom middleware
app.use_middleware(|req, res, next| async move {
    let start = std::time::Instant::now();
    let res = next(req, res).await;
    let duration = start.elapsed();
    println!("Request took: {:?}", duration);
    res
});
```

---

## 🛠️ Advanced Features

### File Upload

Handle multipart uploads with validation and size limits:

```rust
let uploader = Uploader::new(
    UploadConfig::new()
        .destination("./uploads")
        .max_file_size_mb(10)
        .allowed_extensions(vec!["png", "jpg", "pdf"])
);

app.post("/upload", move |req, res| {
    let uploader = uploader.clone();
    async move {
        // Parse multipart form
        let content_type = req.content_type().unwrap_or_default();
        let boundary = parse_boundary(&content_type).unwrap();
        let fields = parse_multipart(req.body(), &boundary).unwrap();

        for field in fields {
            if let Some(filename) = field.filename {
                let file = uploader.upload_single(
                    &field.name,
                    field.data,
                    &filename,
                    &field.content_type.unwrap_or_default()
                ).await?;
                return Ok(res.json(json!({ "file": file.filename })));
            }
        }
        Ok(res.bad_request("No file provided"))
    }
});
```

### Rate Limiting

Protect APIs from excessive requests:

```rust
let config = RateLimiterConfig::new(100, 60)
    .message("Too many requests, please slow down.");

app.use_middleware(rate_limiter(config));
```

### WebSocket Support

Real-time two-way communication:

```rust
use nitrox::websocket::{WsServer, WsMessage};

let ws = WsServer::new();
ws.broadcast(WsMessage::Text("Global notification".into())).await;
```

### Database Integration

Nitrox stays out of your way and is completely database agnostic. You can plug in your favorite database library (SQLx, Diesel, SeaORM, MongoDB, Redis) via your application state:

```rust
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;

struct AppState {
    db: SqlitePool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let pool = SqlitePool::connect("sqlite:app.db").await?;
    let state = Arc::new(AppState { db: pool });

    let app = Nitrox::new();

    let db_state = Arc::clone(&state);
    app.get("/users", move |_req, res| {
        let state = Arc::clone(&db_state);
        async move {
            let users = sqlx::query_as!(User, "SELECT id, name FROM users")
                .fetch_all(&state.db)
                .await
                .unwrap_or_default();
            res.json(json!({ "users": users }))
        }
    });

    app.listen(3000).await
}
```

---

## 🚀 Deployment

### Production Release

```bash
cargo build --release
```

### Dockerfile

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/nitrox /usr/local/bin/
EXPOSE 3000
CMD ["nitrox"]
```

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file.

<div align="center">

**Author: Somen Das ([somen8099@gmail.com](mailto:somen8099@gmail.com))**

⭐ Star on [GitHub](https://github.com/KTBsomen/nitrox)!

</div>
