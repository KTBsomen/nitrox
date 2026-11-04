# 🚀 RustyX

<div align="center">

![RustyX Logo](https://img.shields.io/badge/RustyX-Fast%20Web%20Framework-orange?style=for-the-badge&logo=rust)

[![Crates.io](https://img.shields.io/crates/v/rustyx.svg?style=flat-square)](https://crates.io/crates/rustyx)
[![Documentation](https://docs.rs/rustyx/badge.svg)](https://docs.rs/rustyx)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/github/actions/workflow/status/Mohammad007/rustyx/ci.yml?branch=main&style=flat-square)](https://github.com/Mohammad007/rustyx/actions)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg?style=flat-square)](https://www.rust-lang.org)

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
| 🎯 **ExpressJS-like API** | Familiar interface for JavaScript/Node.js developers |
| ⚡ **Blazingly Fast** | Built on Hyper and Tokio for maximum performance |
| 🔌 **Middleware Support** | Logger, CORS, Rate Limiting, Helmet, Timeout |
| 🗄️ **Multi-Database ORM** | MongoDB, MySQL, PostgreSQL, SQLite support |
| 🌐 **WebSocket Support** | Real-time bidirectional communication |
| 📁 **Static Files** | Serve static assets with MIME type detection |
| 📤 **File Upload** | Multer-like file upload with validation |
| 🔒 **Type-Safe** | Leverage Rust's type system for safer code |
| 📝 **JSON-First** | Designed for building REST APIs |
| 🛡️ **Security** | Built-in Helmet middleware for security headers |
| ⏱️ **Rate Limiting** | Protect APIs from abuse |

---

## 📦 Installation

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Cargo** - Comes with Rust

### Add to Cargo.toml

```toml
[dependencies]
rustyx = "0.1.0"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### With Database Features

```toml
[dependencies]
# SQLite (default)
rustyx = "0.1.0"

# MySQL
rustyx = { version = "0.1.0", features = ["mysql"] }

# PostgreSQL
rustyx = { version = "0.1.0", features = ["postgres"] }

# MongoDB
rustyx = { version = "0.1.0", features = ["mongodb"] }

# All databases
rustyx = { version = "0.1.0", features = ["full"] }
```

### Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `default` | SQLite support | ✅ |
| `mysql` | MySQL database | ❌ |
| `postgres` | PostgreSQL database | ❌ |
| `sqlite` | SQLite database | ✅ |
| `mongodb` | MongoDB database | ❌ |
| `full` | All database drivers | ❌ |

---

## 🚀 Getting Started

### 1. Create a New Project

```bash
cargo new my_api
cd my_api
```

### 2. Add Dependencies

Edit `Cargo.toml`:

```toml
[package]
name = "my_api"
version = "0.1.0"
edition = "2021"

[dependencies]
rustyx = "0.1.0"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 3. Create Your First API

Edit `src/main.rs`:

```rust
use rustyx::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().init();

    // Create application
    let app = RustyX::new();

    // Define routes
    app.get("/", |_req, res| async move {
        res.json(json!({
            "message": "Welcome to RustyX!",
            "version": rustyx::VERSION
        }))
    });

    app.get("/hello/:name", |req, res| async move {
        let name = req.param("name").unwrap_or(&"World".to_string());
        res.json(json!({ "message": format!("Hello, {}!", name) }))
    });

    // Start server
    info!("🚀 Server starting...");
    app.listen(3000).await
}
```

### 4. Run Your Server

```bash
cargo run
```

Visit `http://localhost:3000` in your browser!

---

## 📚 Core Concepts

### Application

The `RustyX` struct is the main entry point:

```rust
use rustyx::prelude::*;

let app = RustyX::new();

// Configure routes
app.get("/path", handler);
app.post("/path", handler);
app.put("/path", handler);
app.delete("/path", handler);
app.patch("/path", handler);

// Add middleware
app.use_middleware(logger());
app.use_middleware(cors("*"));

// Start server
app.listen(3000).await?;
```

### Routing

RustyX supports ExpressJS-style routing with path parameters:

```rust
// Basic routes
app.get("/", handler);
app.post("/users", create_user);

// Path parameters
app.get("/users/:id", |req, res| async move {
    let id = req.param("id").unwrap();
    res.json(json!({ "user_id": id }))
});

// Multiple parameters
app.get("/users/:userId/posts/:postId", |req, res| async move {
    let user_id = req.param("userId").unwrap();
    let post_id = req.param("postId").unwrap();
    res.json(json!({ "user": user_id, "post": post_id }))
});

// Using Router for grouping
let mut api = Router::with_prefix("/api/v1");
api.get("/users", list_users);
api.post("/users", create_user);
app.use_router("/", api);
```

### Request

The `Request` object provides access to request data:

```rust
|req, res| async move {
    // HTTP method
    let method = req.method();
    
    // Request path
    let path = req.path();
    
    // URL parameters (/users/:id)
    let id = req.param("id");
    
    // Query parameters (?page=1&limit=10)
    let page = req.query_param("page");
    let all_query = req.query();
    
    // Parse JSON body
    let user: User = req.json()?;
    
    // Raw body
    let body_bytes = req.body();
    let body_string = req.body_string()?;
    
    // Headers
    let content_type = req.header("content-type");
    let auth_header = req.authorization();
    let token = req.bearer_token();
    
    // Client information
    let ip = req.ip();
    let user_agent = req.user_agent();
    
    // Content checks
    let is_json = req.is_json();
    let accepts_json = req.accepts("application/json");
    
    res.send("OK")
}
```

### Response

The `Response` object provides methods for sending responses:

```rust
|req, res| async move {
    // JSON response
    res.json(json!({ "key": "value" }))
    
    // Text response
    res.send("Hello, World!")
    
    // HTML response
    res.html("<h1>Hello</h1>")
    
    // Status codes
    res.status(201).json(data)
    
    // Common responses
    res.created(data)           // 201
    res.no_content()            // 204
    res.bad_request("message")  // 400
    res.unauthorized()          // 401
    res.forbidden()             // 403
    res.not_found()             // 404
    res.internal_error("msg")   // 500
    
    // Redirects
    res.redirect("/new-location")
    res.redirect_permanent("/moved")
    
    // Headers
    res.header("X-Custom", "value")
    res.content_type("application/xml")
    res.cors("*")
    
    // Cookies
    res.cookie("session", "abc123", CookieOptions::new())
    res.clear_cookie("session")
}
```

### Middleware

Middleware functions process requests before they reach route handlers:

```rust
use rustyx::middleware::*;

let app = RustyX::new();

// Built-in middleware
app.use_middleware(logger());           // Request logging
app.use_middleware(cors("*"));          // CORS headers
app.use_middleware(helmet());           // Security headers
app.use_middleware(timeout(30000));     // 30s timeout
app.use_middleware(request_id());       // Add X-Request-ID
app.use_middleware(response_time());    // Add X-Response-Time

// Rate limiting
let rate_config = RateLimiterConfig::new(100, 60); // 100 req/min
app.use_middleware(rate_limiter(rate_config));

// Custom middleware
app.use_middleware(|req, res, next| async move {
    println!("Before: {} {}", req.method(), req.path());
    let response = next(req, res).await;
    println!("After: {}", response.get_status());
    response
});
```

---

## 🔥 Advanced Features

### File Upload

Handle single and multiple file uploads (similar to Express Multer):

```rust
use rustyx::prelude::*;

let uploader = Uploader::new(
    UploadConfig::new()
        .destination("./uploads")
        .max_file_size_mb(5)
        .allowed_extensions(vec!["png", "jpg", "jpeg", "pdf"])
);

app.post("/upload", move |req, res| {
    let uploader = uploader.clone();
    async move {
        let content_type = req.content_type().unwrap_or_default();
        let boundary = parse_boundary(&content_type).unwrap();
        let fields = parse_multipart(req.body(), &boundary)?;
        
        for field in fields {
            if let Some(filename) = field.filename {
                let file = uploader.upload_single(
                    &field.name,
                    field.data,
                    &filename,
                    &field.content_type.unwrap_or_default()
                ).await?;
                
                return res.json(json!({
                    "filename": file.filename,
                    "size": file.size
                }));
            }
        }
        res.bad_request("No file provided")
    }
});
```

Configuration options:

```rust
UploadConfig::new()
    .destination("./uploads")        // Upload directory
    .max_file_size_mb(10)            // Max 10MB
    .max_files(5)                    // Max 5 files per request
    .images_only()                   // Only images (PNG, JPG, GIF, WebP)
    .documents_only()                // Only docs (PDF, DOC, XLS)
    .allowed_extensions(vec!["png", "pdf"])  // Custom extensions
    .keep_original_name()            // Keep original filename
    .use_uuid()                      // UUID filename (default)
```

📖 **Full documentation:** [docs/UPLOAD.md](docs/UPLOAD.md)

### Rate Limiting

Protect your API from abuse:

```rust
use rustyx::middleware::{rate_limiter, RateLimiterConfig};

// Basic: 100 requests per 60 seconds
app.use_middleware(simple_rate_limit(100, 60));

// Advanced configuration
let config = RateLimiterConfig::new(100, 60)
    .message("Rate limit exceeded. Try again later.")
    .skip(vec!["/health", "/metrics"]);

app.use_middleware(rate_limiter(config));
```

### WebSocket Support

Real-time communication:

```rust
use rustyx::websocket::{WsServer, WsMessage};

let ws_server = WsServer::new();

// Send to specific client
ws_server.send_to(&conn_id, WsMessage::Text("Hello!".into())).await;

// Broadcast to all
ws_server.broadcast(WsMessage::Text("Announcement".into())).await;

// Room-based messaging
ws_server.join_room("chat", conn_id.clone());
ws_server.broadcast_to_room("chat", WsMessage::Text("Chat message".into())).await;
```

### Static File Serving

Serve static files:

```rust
use rustyx::static_files::{static_handler, StaticConfig};

// Basic usage
let config = StaticConfig::new("./public");
app.get("/static/*", static_handler(config));

// With options
let config = StaticConfig::new("./public")
    .index("index.html")
    .max_age(3600)
    .directory_listing(false);
```

### Database Integration

Connect and query databases:

```rust
use rustyx::db::prelude::*;

// Configure SQLite
let config = DatabaseConfig::new(DbDriver::SQLite, "app.db");

// Configure PostgreSQL
let config = DatabaseConfig::new(DbDriver::PostgreSQL, "mydb")
    .host("localhost")
    .port(5432)
    .username("user")
    .password("pass");

// Initialize connection
init_db(config).await?;

// Query builder
let query = QueryBuilder::table("users")
    .select(&["id", "name", "email"])
    .where_eq("active", "true")
    .order_by("created_at", Order::Desc)
    .limit(10)
    .build();
```

---

## 🚀 Deployment

### Production Build

```bash
# Create optimized release build
cargo build --release

# The binary is at ./target/release/my_api
```

### Docker Deployment

Create `Dockerfile`:

```dockerfile
# Build stage
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/my_api /usr/local/bin/
EXPOSE 3000
CMD ["my_api"]
```

Build and run:

```bash
docker build -t my-api .
docker run -p 3000:3000 my-api
```

### Docker Compose

```yaml
version: '3.8'
services:
  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info
      - DATABASE_URL=postgres://user:pass@db:5432/mydb
    depends_on:
      - db
  db:
    image: postgres:15
    environment:
      POSTGRES_USER: user
      POSTGRES_PASSWORD: pass
      POSTGRES_DB: mydb
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

### Environment Variables

```rust
use std::env;

let port: u16 = env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string())
    .parse()
    .unwrap();

let db_url = env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
```

### Systemd Service

Create `/etc/systemd/system/my-api.service`:

```ini
[Unit]
Description=My RustyX API
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/my-api
ExecStart=/opt/my-api/my_api
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=PORT=3000

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable my-api
sudo systemctl start my-api
sudo systemctl status my-api
```

### Nginx Reverse Proxy

```nginx
server {
    listen 80;
    server_name api.example.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}
```

### Cloud Deployment

#### AWS / DigitalOcean / Linode

1. Create a VPS instance
2. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
3. Clone and build your project
4. Configure systemd service
5. Set up Nginx reverse proxy
6. Configure SSL with Let's Encrypt

#### Railway / Render / Fly.io

These platforms auto-detect Rust projects. Just push your code!

```bash
# Fly.io example
flyctl launch
flyctl deploy
```

---

## 📖 API Reference

### RustyX (Application)

| Method | Signature | Description |
|--------|-----------|-------------|
| `new()` | `RustyX::new()` | Create new app |
| `get()` | `.get(path, handler)` | GET route |
| `post()` | `.post(path, handler)` | POST route |
| `put()` | `.put(path, handler)` | PUT route |
| `delete()` | `.delete(path, handler)` | DELETE route |
| `patch()` | `.patch(path, handler)` | PATCH route |
| `use_middleware()` | `.use_middleware(fn)` | Add middleware |
| `use_router()` | `.use_router(path, router)` | Mount router |
| `listen()` | `.listen(port).await` | Start server |

### Request

| Method | Returns | Description |
|--------|---------|-------------|
| `method()` | `&Method` | HTTP method |
| `path()` | `&str` | Request path |
| `param(name)` | `Option<&String>` | URL parameter |
| `query_param(name)` | `Option<&String>` | Query parameter |
| `json<T>()` | `Result<T>` | Parse JSON body |
| `body()` | `&Bytes` | Raw body |
| `header(name)` | `Option<&str>` | Get header |
| `bearer_token()` | `Option<&str>` | Bearer token |
| `ip()` | `IpAddr` | Client IP |

### Response

| Method | Description |
|--------|-------------|
| `.status(code)` | Set status code |
| `.json(data)` | Send JSON |
| `.send(text)` | Send text |
| `.html(html)` | Send HTML |
| `.redirect(url)` | Redirect |
| `.header(name, value)` | Set header |
| `.cookie(name, value, opts)` | Set cookie |

---

## 📝 Examples

### REST API with CRUD

```rust
use rustyx::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: Option<String>,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = RustyX::new();
    
    app.use_middleware(logger());
    app.use_middleware(cors("*"));

    // List users
    app.get("/api/users", |_req, res| async move {
        res.json(json!({ "users": [] }))
    });

    // Get user
    app.get("/api/users/:id", |req, res| async move {
        let id = req.param("id").unwrap();
        res.json(json!({ "id": id }))
    });

    // Create user
    app.post("/api/users", |req, res| async move {
        let user: User = req.json()?;
        res.created(user)
    });

    // Update user
    app.put("/api/users/:id", |req, res| async move {
        let user: User = req.json()?;
        res.json(user)
    });

    // Delete user
    app.delete("/api/users/:id", |req, res| async move {
        res.no_content()
    });

    app.listen(3000).await
}
```

### With Authentication

```rust
app.use_middleware(|req, res, next| async move {
    // Skip auth for public routes
    if req.path() == "/login" || req.path() == "/health" {
        return next(req, res).await;
    }

    // Check authorization
    match req.bearer_token() {
        Some(token) if validate_token(token) => next(req, res).await,
        _ => res.unauthorized(),
    }
});
```

---

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push: `git push origin feature/amazing-feature`
5. Open a Pull Request

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file.

---

<div align="center">

**Made with ❤️ by the RustyX Team**

⭐ Star us on [GitHub](https://github.com/Mohammad007/rustyx)!

[Report Bug](https://github.com/Mohammad007/rustyx/issues) •
[Request Feature](https://github.com/Mohammad007/rustyx/issues) •
[Documentation](https://docs.rs/rustyx)

</div>
