# 📦 Installation Guide

This guide will help you install and set up RustyX for your project.

## Prerequisites

### Rust Installation

RustyX requires **Rust 1.70** or higher. Install Rust using rustup:

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# Download and run rustup-init.exe from https://rustup.rs
```

Verify installation:

```bash
rustc --version
cargo --version
```

### Optional: Database Drivers

For database features, you may need:

- **SQLite**: Usually bundled, no extra installation needed
- **PostgreSQL**: `libpq-dev` (Linux) or PostgreSQL installation
- **MySQL**: `libmysqlclient-dev` (Linux) or MySQL installation
- **MongoDB**: MongoDB server running locally or remotely

---

## Quick Installation

### Step 1: Create New Project

```bash
cargo new my_rustyx_app
cd my_rustyx_app
```

### Step 2: Add Dependencies

Edit `Cargo.toml`:

```toml
[package]
name = "my_rustyx_app"
version = "0.1.0"
edition = "2021"

[dependencies]
rustyx = "0.1.0"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["fmt", "env-filter"] }
```

### Step 3: Create Main File

Replace `src/main.rs`:

```rust
use rustyx::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let app = RustyX::new();

    app.get("/", |_req, res| async move {
        res.json(json!({
            "message": "Hello from RustyX!",
            "version": rustyx::VERSION
        }))
    });

    app.listen(3000).await
}
```

### Step 4: Run

```bash
cargo run
```

---

## Installation with Database Support

### SQLite (Default)

```toml
rustyx = "0.1.0"
```

### PostgreSQL

```toml
rustyx = { version = "0.1.0", features = ["postgres"] }
```

On Ubuntu/Debian:
```bash
sudo apt install libpq-dev
```

### MySQL

```toml
rustyx = { version = "0.1.0", features = ["mysql"] }
```

On Ubuntu/Debian:
```bash
sudo apt install libmysqlclient-dev
```

### MongoDB

```toml
rustyx = { version = "0.1.0", features = ["mongodb"] }
```

### All Databases

```toml
rustyx = { version = "0.1.0", features = ["full"] }
```

---

## Project Templates

### Minimal API

```toml
[dependencies]
rustyx = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

### Full-Featured API

```toml
[dependencies]
rustyx = { version = "0.1.0", features = ["postgres"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["fmt", "env-filter"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dotenv = "0.15"
```

---

## Troubleshooting

### Common Issues

#### 1. Compilation Errors

Make sure you have the latest Rust:
```bash
rustup update
```

#### 2. Missing Native Dependencies

On Ubuntu/Debian:
```bash
sudo apt install build-essential pkg-config libssl-dev
```

On macOS:
```bash
xcode-select --install
```

#### 3. Port Already in Use

```bash
# Find process using port 3000
lsof -i :3000

# Kill process
kill -9 <PID>
```

#### 4. Slow Compilation

Enable faster linking in `.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

---

## Verifying Installation

Run the health check:

```bash
cargo run
```

Then test with curl:

```bash
curl http://localhost:3000
```

Expected output:
```json
{"message":"Hello from RustyX!","version":"0.1.0"}
```

---

## Next Steps

- [Quick Start Guide](./QUICKSTART.md)
- [API Documentation](./API.md)
- [Middleware Guide](./MIDDLEWARE.md)
- [Database Guide](./DATABASE.md)
- [Deployment Guide](./DEPLOYMENT.md)
