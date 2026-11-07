# Installation & Setup Guide

This guide covers setting up Nitrox for your project, configuring dependencies, and integrating external database drivers.

---

## Prerequisites

Nitrox requires **Rust 1.75** or higher. Install Rust using rustup:

```bash
# Linux / macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# Download and run rustup-init.exe from https://rustup.rs
```

Verify your installation:

```bash
rustc --version
cargo --version
```

---

## Quick Installation

### Step 1: Create a New Project

```bash
cargo new my_nitrox_app
cd my_nitrox_app
```

### Step 2: Add Dependencies to `Cargo.toml`

```toml
[package]
name = "my_nitrox_app"
version = "0.1.0"
edition = "2021"

[dependencies]
nitrox = "0.0.3"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Step 3: Write Your Server (`src/main.rs`)

```rust
use nitrox::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = Nitrox::new();

    app.get("/", |_req, res| async move {
        res.json(json!({
            "message": "Hello from Nitrox!",
            "version": nitrox::VERSION
        }))
    });

    println!("Server running on http://127.0.0.1:3000");
    app.listen(3000).await
}
```

### Step 4: Run the Server

```bash
cargo run
```

Test with curl:
```bash
curl http://127.0.0.1:3000
```

---

## Database Integration: Database-Agnostic Architecture

**Nitrox is deliberately database-agnostic.** It does not bundle database drivers, hide queries behind a forced ORM, or introduce unnecessary feature flags. 

Instead, you use official, high-performance community crates directly and pass the thread-safe connection pools into your application state struct via [`Nitrox::with_state(state)`](file:///c:/Users/somen/myProjects/rustyx/src/app.rs#L220-L265).

### Example 1: PostgreSQL with SQLx

Add `sqlx` to your `Cargo.toml`:

```toml
[dependencies]
nitrox = "0.0.3"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres"] }
```

In `src/main.rs`:

```rust
use nitrox::prelude::*;
use sqlx::PgPool;

struct AppState {
    db: PgPool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let pool = PgPool::connect("postgres://user:password@localhost:5432/mydb")
        .await
        .expect("Failed to connect to PostgreSQL");

    let state = AppState { db: pool };
    let app = Nitrox::with_state(state);

    app.get("/users", |req, res| async move {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&req.state.db)
            .await
            .unwrap_or((0,));

        res.json(json!({ "total_users": count.0 }))
    });

    app.listen(3000).await
}
```

---

### Example 2: MongoDB

Add `mongodb` to your `Cargo.toml`:

```toml
[dependencies]
nitrox = "0.0.3"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
mongodb = "3.1"
```

In `src/main.rs`:

```rust
use nitrox::prelude::*;
use mongodb::{Client, Database};

struct AppState {
    mongo: Database,
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .expect("Failed to connect to MongoDB");
    let database = client.database("my_app");

    let state = AppState { mongo: database };
    let app = Nitrox::with_state(state);

    app.get("/health", |req, res| async move {
        let db_name = req.state.mongo.name();
        res.json(json!({ "status": "ok", "database": db_name }))
    });

    app.listen(3000).await
}
```

---

### Example 3: Redis (Cache & Rate Limiting)

Add `redis` to your `Cargo.toml`:

```toml
[dependencies]
nitrox = "0.0.3"
tokio = { version = "1", features = ["full"] }
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }
```

In `src/main.rs`:

```rust
use nitrox::prelude::*;
use redis::aio::ConnectionManager;

struct AppState {
    redis: ConnectionManager,
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = redis::Client::open("redis://127.0.0.1:6379/")
        .expect("Invalid Redis URL");
    let manager = ConnectionManager::new(client)
        .await
        .expect("Failed to connect to Redis");

    let state = AppState { redis: manager };
    let app = Nitrox::with_state(state);

    app.get("/stats", |req, res| async move {
        let mut conn = req.state.redis.clone();
        let visits: i64 = redis::cmd("INCR")
            .arg("site_visits")
            .query_async(&mut conn)
            .await
            .unwrap_or(0);

        res.json(json!({ "site_visits": visits }))
    });

    app.listen(3000).await
}
```

---

## Troubleshooting

### 1. Port Already in Use

```bash
# Check process using port 3000 (Linux / macOS)
lsof -i :3000
kill -9 <PID>

# Windows PowerShell
Get-NetTCPConnection -LocalPort 3000
Stop-Process -Id <PID>
```

### 2. Missing C Toolchain (When using jemalloc or OpenSSL)

- **Ubuntu / Debian:** `sudo apt install build-essential pkg-config libssl-dev`
- **macOS:** `xcode-select --install`
- **Windows:** Install Visual Studio C++ Build Tools

---

## Next Steps

- [**Beginner's Guide & Express Migration**](./BEGINNER_GUIDE.md) - Complete tutorial and Rust gotchas
- [**Application State & Databases**](./STATE.md) - Deep dive into state patterns and avoiding lock contention
- [**Quick Start Guide**](./QUICKSTART.md) - Build a working REST API in 5 minutes
- [**Middleware Master Guide**](./MIDDLEWARE.md) - 4-tier middleware lifecycle and recipes
- [**Multipart File Uploads**](./UPLOAD.md) - Multer-style file streaming
- [**Production Deployment**](./DEPLOYMENT.md) - Docker, systemd, and reverse proxy configuration
