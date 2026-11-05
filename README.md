# Nitrox

<div align="center">

![Nitrox](https://img.shields.io/badge/Nitrox-Web%20Framework-blue?style=for-the-badge&logo=rust)

[![Crates.io](https://img.shields.io/crates/v/nitrox.svg?style=flat-square)](https://crates.io/crates/nitrox)
[![Documentation](https://docs.rs/nitrox/badge.svg)](https://docs.rs/nitrox)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](https://opensource.org/licenses/MIT)

[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-blue.svg?style=flat-square)](https://www.rust-lang.org)

**A high-performance, minimalist web framework for Rust inspired by ExpressJS.**

[Getting Started](#getting-started) •
[Installation](#installation) •
[State Management](#application-state-management) •
[Core Concepts](#core-concepts) •
[Benchmarks](#benchmarks) •
[Deployment](#deployment) •
[License](#license)

</div>

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
- [Getting Started](#getting-started)
- [Application State Management](#application-state-management)
  - [Zero-Clone State Architecture](#zero-clone-state-architecture)
  - [State Type Requirements (`Send + Sync + 'static`)](#state-type-requirements-send--sync--static)
  - [Permitted and Recommended State Types](#permitted-and-recommended-state-types)
  - [Disallowed Types and Compile-Time Restrictions](#disallowed-types-and-compile-time-restrictions)
  - [Runtime Concurrency Pitfalls and Anti-Patterns](#runtime-concurrency-pitfalls-and-anti-patterns)
- [Core Concepts](#core-concepts)
  - [Application](#application)
  - [Routing and Modular Sub-Routers](#routing-and-modular-sub-routers)
  - [Request](#request)
  - [Response](#response)
  - [Middleware](#middleware)
- [Advanced Features](#advanced-features)
  - [Database Integration](#database-integration)
  - [File Upload](#file-upload)
  - [Rate Limiting](#rate-limiting)
  - [WebSocket Support](#websocket-support)
  - [Static Files](#static-file-serving)
- [Benchmarks](#benchmarks)
- [Deployment](#deployment)
- [License](#license)

---

## Overview

Nitrox bridges the gap between the productivity and ergonomic simplicity of Node.js/Express and the raw execution speed, safety, and memory efficiency of Rust.

In traditional Rust web frameworks, sharing state across route handlers typically requires repetitive manual cloning (`let state = state.clone()`) across both outer setup scopes and inner asynchronous closures. Nitrox eliminates this friction entirely by providing an Express-like `req.state` interface backed by lock-free atomic references (`Arc<S>`), enabling zero-clone handler ergonomics with over 65,000 to 88,000 requests per second in real-world HTTP socket benchmarks.

---

## Features

| Feature | Description |
|---|---|
| **ExpressJS-Style API** | Familiar, intuitive interface for JavaScript, TypeScript, and Go backend developers |
| **Zero-Clone State** | First-class application state injection directly accessible on `req.state` |
| **High Performance** | Built directly on Hyper 1.x and Tokio with sub-millisecond median response times |
| **Modular Routing** | Hierarchical sub-routers mounted at path prefixes (`app.use_router`) |
| **Middleware Pipeline** | Chainable request/response middleware including Logger, CORS, Helmet, and Rate Limiting |
| **Database Agnostic** | Seamless integration with SQLx, Diesel, SeaORM, MongoDB, and Redis |
| **Multipart File Upload** | Multer-style file uploads with MIME validation and disk storage |
| **WebSocket Support** | Built-in room management and broadcasting for real-time applications |
| **Static File Server** | High-throughput static asset streaming with automatic MIME detection |
| **Type-Safe Validation** | First-class Serde integration for JSON parsing and schema validation |

---

## Installation

### Prerequisites

- **Rust 1.75+** - [Install Rust](https://rustup.rs/)
- **Cargo** - Standard Rust package manager

### Add to Cargo.toml

```toml
[dependencies]
nitrox = "0.0.2"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## Getting Started

### Minimal Stateless Example

```rust
use nitrox::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Nitrox::new();

    // Global middleware
    app.use_middleware(logger());
    app.use_middleware(cors("*"));

    // Route handlers
    app.get("/", |_req, res| async move {
        res.json(json!({ "status": "operational" }))
    });

    app.get("/users/:id", |req, res| async move {
        let id = req.param("id").unwrap_or_default();
        res.json(json!({ "user_id": id }))
    });

    app.listen(3000).await
}
```

Run the application:

```bash
cargo run
```

---

## Application State Management

### Zero-Clone State Architecture

Enterprise web services require shared dependencies: database connection pools, cache clients, metrics counters, and configuration.

In typical Rust frameworks, closures capturing external references require double-cloning:

```rust
// The Traditional Rust Problem (Double-Cloning Boilerplate)
let db_clone = db.clone();
app.get("/users", move |req, res| {
    let db = db_clone.clone(); // Cloned again for every incoming request closure!
    async move {
        let users = db.get_users().await;
        res.json(users)
    }
});
```

Nitrox solves this at the architectural level. By passing your application state once to `Nitrox::with_state(state)`, Nitrox automatically manages the `Arc<S>` lifecycle and injects state directly into every incoming `Request<S>`:

```rust
use nitrox::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

struct AppState {
    environment: String,
    request_counter: AtomicU64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState {
        environment: "production".to_string(),
        request_counter: AtomicU64::new(0),
    };

    // Initialize application with shared state
    let app = Nitrox::with_state(state);

    // Handlers access state directly via `req.state` with zero cloning
    app.get("/status", |req, res| async move {
        let count = req.state.request_counter.fetch_add(1, Ordering::Relaxed);
        let env = &req.state.environment;

        res.json(json!({
            "environment": env,
            "requests_served": count
        }))
    });

    app.listen(3000).await
}
```

---

### State Type Requirements (`Send + Sync + 'static`)

Because Nitrox processes HTTP requests concurrently across a multi-threaded Tokio runtime pool, any type passed to `Nitrox::with_state(state)` must satisfy three trait bounds:

```rust
impl<S: Send + Sync + 'static> Nitrox<S> { ... }
```

1. **`Send`**: The state can be safely moved to another thread (e.g. dispatched to worker threads).
2. **`Sync`**: Immutable references `&S` can be concurrently shared across multiple threads without data races.
3. **`'static`**: The state must own all of its data or hold references that live for the entire duration of the program. Temporary stack references are forbidden.

---

### Permitted and Recommended State Types

| State Category | Recommended Types | Usage Pattern |
|---|---|---|
| **Database Connection Pools** | `sqlx::PgPool`, `sqlx::MySqlPool`, `sqlx::SqlitePool`, `deadpool_redis::Pool`, `bb8::Pool` | Thread-safe connection pools handle internal synchronization. Store directly on your state struct. |
| **HTTP Client Singletons** | `reqwest::Client` | Maintains internal connection pools and TLS session caches. Reusable across the entire process. |
| **Concurrent Caches & Maps** | `dashmap::DashMap<K, V>`, `moka::future::Cache<K, V>`, `parking_lot::RwLock<HashMap<K, V>>` | Sharded or read-optimized concurrency for fast in-memory lookups. |
| **Background Channels** | `tokio::sync::mpsc::Sender<T>`, `tokio::sync::broadcast::Sender<T>` | Enqueue background tasks, log events, or audit trails to asynchronous worker loops. |
| **Atomic Metrics** | `std::sync::atomic::{AtomicU64, AtomicBool, AtomicUsize}` | Lock-free counters, feature flags, and uptime trackers. |
| **Dynamic Configuration** | `arc_swap::ArcSwap<Config>` | Allows zero-lock atomic configuration reloads at runtime without stopping the server. |

---

### Disallowed Types and Compile-Time Restrictions

Rust's compiler prevents invalid memory patterns before your code can run. The following types **cannot** be passed into `Nitrox::with_state`:

#### 1. Single-Threaded Reference Counters (`Rc<T>`)
- **Reason:** `Rc` uses non-atomic reference counting. Sharing it across worker threads causes data races and memory corruption.
- **Compiler Rejection:** `the trait 'Send' is not implemented for 'Rc<T>'`
- **Correction:** Use `Arc<T>` instead.

#### 2. Single-Threaded Interior Mutability (`RefCell<T>`, `Cell<T>`)
- **Reason:** `RefCell` checks borrow rules at runtime on a single thread. It is neither `Send` nor `Sync`.
- **Compiler Rejection:** `the trait 'Sync' is not implemented for 'RefCell<T>'`
- **Correction:** Use `parking_lot::RwLock<T>` (for synchronous operations) or `tokio::sync::RwLock<T>` (for async operations).

#### 3. Raw Pointers (`*const T`, `*mut T`)
- **Reason:** Raw pointers do not implement `Send` or `Sync` without unsafe manual wrappers.
- **Correction:** Use safe smart pointers (`Arc<T>`, `Box<T>`).

#### 4. Temporary References (Non-`'static` Lifetimes)
- **Reason:** `AppState<'a>` holding a borrowed `&'a str` or `&'a Database` cannot outlive the function that created it.
- **Compiler Rejection:** `lifetime ''static' required`
- **Correction:** Own your data (`String`, `Vec<u8>`) or use `'static` string literals (`&'static str`).

---

### Runtime Concurrency Pitfalls and Anti-Patterns

While the compiler enforces memory safety, adhering to good concurrency practices avoids runtime performance degradation:

#### Anti-Pattern 1: Holding `std::sync::Mutex` Across an `.await`
```rust
// INCORRECT: Standard library MutexGuard across an await boundary
let mut lock = req.state.std_mutex.lock().unwrap();
let data = fetch_remote_api().await; // Compiler error or worker thread starvation!
lock.push(data);
```
- **Why this is problematic:** `std::sync::MutexGuard` is not `Send`. Holding a synchronous lock across an `.await` boundary blocks the underlying OS thread, preventing Tokio from scheduling other tasks.
- **Solution:**
  - If the critical section does not involve async I/O, acquire and drop the lock synchronously before or after `.await`.
  - If a lock must be held across `.await`, use `tokio::sync::Mutex`.
  - For read-heavy operations, use `parking_lot::RwLock`.

#### Anti-Pattern 2: Un-Pooled Single Database Connections
```rust
// INCORRECT: Raw SQLite connection without a pool
struct AppState {
    conn: rusqlite::Connection, // Fails `Sync` requirement
}
```
- **Why this is problematic:** A single connection cannot execute concurrent queries simultaneously.
- **Solution:** Use connection pool managers such as `r2d2`, `deadpool`, or `sqlx::Pool`.

#### Anti-Pattern 3: Blocking CPU-Intensive Tasks on Async Workers
```rust
// INCORRECT: Blocking CPU work directly in async route handler
app.get("/hash", |req, res| async move {
    let result = heavy_pbkdf2_computation(); // Freezes the Tokio worker thread
    res.json(result)
});
```
- **Solution:** Offload CPU-bound or blocking synchronous filesystem operations using `tokio::task::spawn_blocking`:
```rust
app.get("/hash", |req, res| async move {
    let result = tokio::task::spawn_blocking(|| heavy_pbkdf2_computation()).await?;
    res.json(result)
});
```

---

## Core Concepts

### Application

The `Nitrox` instance orchestrates settings, middleware registration, sub-routers, and server lifecycle:

```rust
use nitrox::prelude::*;

let app = Nitrox::new();

// Configuration
app.set("env", "production");
app.set("trust_proxy", true);

// Start server
app.listen(3000).await?;
```

---

### Routing and Modular Sub-Routers

Nitrox supports all HTTP methods (`get`, `post`, `put`, `delete`, `patch`) and provides modular sub-routers for clean multi-file architectures:

```rust
// routes/users.rs
pub fn router() -> Router<AppState> {
    let mut r = Router::with_state();

    r.get("", |req, res| async move {
        let (users, total) = req.state.user_service.get_all().await;
        res.json(json!({ "users": users, "total": total }))
    });

    r.get("/:id", |req, res| async move {
        let id = req.param("id").unwrap_or_default();
        let user = req.state.user_service.get_by_id(&id).await;
        res.json(user)
    });

    r
}

// main.rs
let app = Nitrox::with_state(state);
app.use_router("/api/v1/users", routes::users::router());
```

---

### Request

The `Request<S>` struct provides zero-cost access to all HTTP metadata:

```rust
app.post("/api/users/:id", |req, res| async move {
    // Route parameter (/api/users/:id)
    let id = req.param("id").unwrap_or_default();

    // Query parameter (?page=2&limit=10)
    let page: u32 = req.query_param("page").and_then(|v| v.parse().ok()).unwrap_or(1);

    // Request headers
    let auth_header = req.header("authorization");
    let bearer = req.bearer_token();

    // Shared Application State
    let service = &req.state.user_service;

    // Deserialized JSON body
    let payload: CreateUserDto = req.json()?;

    res.status(201).json(payload)
});
```

---

### Response

The fluent, chainable `Response` builder supports standard HTTP semantics:

```rust
app.get("/example", |_req, res| async move {
    res.status(200)
        .header("X-Powered-By", "Nitrox")
        .cookie("session_id", "abc123xyz", CookieOptions::new().http_only(true).secure(true))
        .json(json!({ "status": "success" }))
});
```

---

### Middleware

Built-in and custom middleware execute in an ordered pipeline:

```rust
// Standard middleware
app.use_middleware(logger());
app.use_middleware(cors("*"));
app.use_middleware(helmet());
app.use_middleware(timeout(15000)); // 15 second timeout

// Custom middleware with access to state
app.use_middleware(|req, res, next| async move {
    let start = std::time::Instant::now();
    let res = next(req, res).await;
    let elapsed = start.elapsed();
    res.header("X-Response-Time-Micros", &elapsed.as_micros().to_string())
});
```

---

## Advanced Features

### Database Integration

Nitrox does not force an ORM onto your project. Connect using your preferred database driver and pass the pool into your application state:

```rust
use nitrox::prelude::*;
use sqlx::sqlite::SqlitePool;

struct AppState {
    db: SqlitePool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let pool = SqlitePool::connect("sqlite:app.db").await?;
    let state = AppState { db: pool };

    let app = Nitrox::with_state(state);

    app.get("/users", |req, res| async move {
        let users = sqlx::query!("SELECT id, name FROM users")
            .fetch_all(&req.state.db)
            .await
            .unwrap_or_default();

        res.json(json!({ "users": users.len() }))
    });

    app.listen(3000).await
}
```

---

### File Upload

Handle multipart file uploads with automatic validation, size limits, and disk streaming:

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
                return Ok(res.json(json!({ "filename": file.filename, "size": file.size })));
            }
        }
        Ok(res.bad_request("No file provided"))
    }
});
```

---

### Rate Limiting

Protect endpoints from abuse using the sliding-window rate limiter:

```rust
let config = RateLimiterConfig::new(100, 60) // 100 requests per 60 seconds
    .message("Rate limit exceeded. Please try again later.");

app.use_middleware(rate_limiter(config));
```

---

### WebSocket Support

Broadcast messages and manage real-time communication rooms:

```rust
use nitrox::websocket::{WsServer, WsMessage};

let ws = WsServer::new();
ws.broadcast(WsMessage::Text("Service update notice".into())).await;
```

---

### Static File Serving

Stream static files with automatic MIME type resolution and directory protection:

```rust
let static_cfg = StaticConfig::new("./public")
    .index_file("index.html")
    .cache_control("public, max-age=86400");

app.use_router("/assets", static_handler(static_cfg));
```

---

## Benchmarks

Stress tests were conducted on real HTTP network loopback sockets using [Bombardier](https://github.com/codesenberg/bombardier) in release mode (`opt-level = 3`, `lto = true`).

### Bombardier HTTP Load Test (50,000 to 100,000 Requests)

| Benchmark Target | Concurrency | Total Requests | Requests/Sec (Avg) | Latency (Mean) | Success Rate | Throughput |
|---|---|---|---|---|---|---|
| **Health Check (`/health`)** | 50 conns | 50,000 | **69,583.46 req/s** (Peak: 80,928) | **710.76 µs** | 100.00% (50k/50k) | 22.01 MB/s |
| **Enterprise Users API (`/api/v1/users`)** | 50 conns | 50,000 | **67,383.23 req/s** (Peak: 80,251) | **741.80 µs** | 100.00% (50k/50k) | 44.70 MB/s |
| **High Concurrency Load (`/api/v1/users`)** | 100 conns | 100,000 | **69,239.62 req/s** (Peak: 88,057) | **1.44 ms** | 100.00% (100k/100k) | 46.42 MB/s |

*Hardware context: Standard windows laptop environment. Endpoint `/api/v1/users` includes sub-router prefix matching, query string parsing, UUID request ID middleware, memory lock lookups, pagination, and dynamic JSON serialization.*

Detailed benchmark reproducible scripts are available under [`examples/state_benchmark`](examples/state_benchmark) and [`examples/modular_api`](examples/modular_api).
### Benchmarks With Axum Express.js Fastify
https://github.com/KTBsomen/nitrox-benchmarks

---

## Deployment

### Release Build

```bash
cargo build --release
```

### Production Dockerfile

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

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

<div align="center">

**Maintainer: Somen Das ([somen8099@gmail.com](mailto:somen8099@gmail.com))**

[GitHub Repository](https://github.com/KTBsomen/nitrox)

</div>


