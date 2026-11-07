# Nitrox Documentation

<div align="center">

**A high-performance, minimalist web framework for Rust inspired by ExpressJS.**

[![Crates.io](https://img.shields.io/crates/v/nitrox.svg?style=flat-square)](https://crates.io/crates/nitrox)
[![Documentation](https://docs.rs/nitrox/badge.svg)](https://docs.rs/nitrox)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-blue.svg?style=flat-square)](https://www.rust-lang.org)

</div>

---

## Welcome

Nitrox bridges the gap between the developer ergonomics and productivity of **Node.js / Express** and the raw execution speed, type safety, and memory efficiency of **Rust**.

Whether you are an experienced Rustacean or a JavaScript/TypeScript developer writing your very first Rust backend, Nitrox is engineered to eliminate boilerplate and give you a familiar, joyful development experience.

---

## Explore the Guides

| Guide | Description |
|---|---|
| [**Beginner's Guide & Express Migration**](BEGINNER_GUIDE.md) | **Start here!** Complete tutorial, Express-to-Rust conversion, every function explained, and 7 critical Rust gotchas. |
| [**Installation & Setup Guide**](INSTALLATION.md) | How to add Nitrox and integrate external database pools (SQLx, MongoDB, Redis). |
| [**Middleware Master Guide**](MIDDLEWARE.md) | Exhaustive guide to Global, Router-level, Route-level, and Fluent RouteBuilder middleware with FIFO order and short-circuiting. |
| [**Application State & Databases**](STATE.md) | How to safely share state (PostgreSQL, MongoDB, Redis) without mutex deadlocks or runtime cloning. |
| [**Quick Start Guide**](QUICKSTART.md) | Build a working REST API with Nitrox in under 5 minutes. |
| [**Multipart File Uploads**](UPLOAD.md) | High-speed file uploads, streaming disk storage, and MIME type validation. |
| [**Production Deployment**](DEPLOYMENT.md) | Docker containers, systemd services, reverse proxies, and production tuning. |
| [**AI Context Reference (llms.txt)**](llms.txt) | Machine-readable framework reference formatted specifically for AI coding assistants. |
| [**AI Full Manual (llms-full.txt)**](llms-full.txt) | Complete single-file documentation export for instant AI ingestion. |

---

## Express vs Nitrox at a Glance

### Express.js (Node.js)
```javascript
const express = require('express');
const app = express();

app.use(express.json());

app.get('/users/:id', (req, res) => {
    const id = req.params.id;
    res.json({ user_id: id });
});

app.listen(3000, () => console.log('Listening on 3000'));
```

### Nitrox (Rust)
```rust
use nitrox::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Nitrox::new();

    app.get("/users/:id", |req, res| async move {
        let id = req.param("id").unwrap_or("unknown");
        res.json(serde_json::json!({ "user_id": id }))
    });

    app.listen(3000).await?;
    Ok(())
}
```

---

## Core Capabilities

- **Zero-Clone Application State:** Access databases and caches via `req.state` with zero manual closure captures.
- **Express-Style Router Architecture:** Modular sub-routers mounted at prefixes with `app.mount("/prefix", router)` and `router.use_middleware(mw)`.
- **4-Tier Middleware System:** Global, Router-level, Route-level (`get_with`), and Fluent RouteBuilder (`router.route("/path")`).
- **High Concurrency:** Built directly on Hyper 1.x and Tokio; benchmarks show sustained throughput above **130,000 to 180,000+ requests per second**.
- **First-Class Error Handling:** Safe early returns with `?`, `let else`, and typed Serde parsing.
- **Built-in Batteries:** CORS, Helmet security headers, rate limiting, request logging, and static asset streaming.

