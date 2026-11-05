# Application State Management Guide

Nitrox provides type-safe, zero-clone shared application state through `Nitrox::with_state(state)` and `req.state`.

This guide covers architecture, trait requirements, permitted types, compile-time restrictions, and concurrency best practices.

---

## Table of Contents

- [Motivation and Architecture](#motivation-and-architecture)
- [Quick Start](#quick-start)
- [Trait Bounds (`Send + Sync + 'static`)](#trait-bounds-send--sync--static)
- [Permitted State Types](#permitted-state-types)
- [Disallowed Types and Compile-Time Errors](#disallowed-types-and-compile-time-errors)
- [Concurrency Rules and Runtime Anti-Patterns](#concurrency-rules-and-runtime-anti-patterns)
  - [Rule 1: Never hold `std::sync::Mutex` across `.await`](#rule-1-never-hold-stdsyncmutex-across-await)
  - [Rule 2: Always pool external connections](#rule-2-always-pool-external-connections)
  - [Rule 3: Offload CPU-heavy tasks to `spawn_blocking`](#rule-3-offload-cpu-heavy-tasks-to-spawn_blocking)
- [Modular Routers and Sub-Routers](#modular-routers-and-sub-routers)
- [Production Reference Example](#production-reference-example)

---

## Motivation and Architecture

In asynchronous Rust web development, handlers are closures spawned across a multi-threaded worker pool. When handlers need access to shared services (such as databases, Redis clients, or configuration), standard Rust closures require capturing clones:

```rust
// Traditional Rust Problem: Double-Cloning Boilerplate
let db_outer = db.clone();
app.get("/users", move |req, res| {
    let db_inner = db_outer.clone(); // Cloned again on every request execution
    async move {
        let users = db_inner.fetch_users().await;
        res.json(users)
    }
});
```

In complex applications with dozens of routes and controllers, this pattern creates excessive boilerplate, easy-to-miss performance regressions, and high cognitive load.

### How Nitrox Solves It

1. When you instantiate your application with `Nitrox::with_state(state)`, Nitrox wraps your state in an immutable `Arc<S>` internally.
2. During server startup (`app.listen(port)`), the state reference is frozen into an immutable `ServerState<S>`.
3. Every incoming connection injects a cheap pointer clone into `Request<S>`.
4. Handlers and middlewares access state directly via `req.state`, completely eliminating closure captures and controller cloning.

---

## Quick Start

```rust
use nitrox::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

struct AppState {
    environment: String,
    visits: AtomicUsize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState {
        environment: "production".to_string(),
        visits: AtomicUsize::new(0),
    };

    let app = Nitrox::with_state(state);

    app.get("/", |req, res| async move {
        let count = req.state.visits.fetch_add(1, Ordering::Relaxed);
        res.json(json!({
            "status": "ok",
            "environment": req.state.environment,
            "requests_served": count
        }))
    });

    app.listen(3000).await
}
```

---

## Trait Bounds (`Send + Sync + 'static`)

Any type `S` passed into `Nitrox::with_state` must satisfy:

```rust
impl<S: Send + Sync + 'static> Nitrox<S> { ... }
```

### 1. `Send`
- **Meaning:** Ownership of the state can be safely transferred across thread boundaries.
- **Why Required:** Tokio distributes incoming TCP connections and asynchronous tasks across multiple operating system threads.

### 2. `Sync`
- **Meaning:** Concurrent immutable references (`&S`) can be safely read simultaneously from multiple threads without data races.
- **Why Required:** Multiple route handlers process concurrent HTTP requests in parallel, all reading from the same shared `Arc<S>`.

### 3. `'static`
- **Meaning:** The type contains no short-lived borrowed references tied to a temporary stack frame.
- **Why Required:** The HTTP server runs indefinitely; all state fields must be owned values or references with `'static` lifetime.

---

## Permitted State Types

The following types satisfy `Send + Sync + 'static` and are recommended for production systems:

| Category | Type | Purpose |
|---|---|---|
| **Database Pools** | `sqlx::PgPool`, `sqlx::MySqlPool`, `sqlx::SqlitePool` | Thread-safe connection pooling for relational databases. |
| **Cache Pools** | `deadpool_redis::Pool`, `bb8_redis::RedisConnectionManager` | Reusable Redis connections for caching and session management. |
| **HTTP Clients** | `reqwest::Client` | Shared connection pool, keep-alive connections, and TLS session caching. |
| **Concurrent Maps** | `dashmap::DashMap<K, V>` | High-throughput concurrent key-value store with shard-level locking. |
| **Read-Heavy Caches** | `parking_lot::RwLock<HashMap<K, V>>`, `moka::future::Cache` | Multi-reader single-writer synchronized memory storage. |
| **Atomic Primitives** | `std::sync::atomic::{AtomicU64, AtomicBool, AtomicUsize}` | High-performance lock-free counters, feature flags, and statistics. |
| **Channels** | `tokio::sync::mpsc::Sender<T>`, `tokio::sync::broadcast::Sender<T>` | Asynchronous message passing to background event processing loops. |
| **Configuration** | `arc_swap::ArcSwap<Config>`, immutable structs | Zero-lock atomic configuration reloads at runtime. |

---

## Disallowed Types and Compile-Time Errors

If a type violates Rust's concurrency rules, the Rust compiler rejects it at compile time:

### 1. Single-Threaded Reference Counters (`Rc<T>`)
- **Problem:** `Rc<T>` uses non-atomic reference counting. Cloning or dropping `Rc<T>` across threads causes data races and memory corruption.
- **Compiler Error:** `the trait 'Send' is not implemented for 'Rc<T>'`
- **Correction:** Replace `Rc<T>` with `Arc<T>`.

### 2. Single-Threaded Interior Mutability (`RefCell<T>`, `Cell<T>`)
- **Problem:** `RefCell<T>` enforces borrow rules dynamically at runtime on a single thread. It is not thread-safe.
- **Compiler Error:** `the trait 'Sync' is not implemented for 'RefCell<T>'`
- **Correction:** Use `parking_lot::RwLock<T>` (for synchronous in-memory read/write) or `tokio::sync::RwLock<T>` (for async operations).

### 3. Raw Pointers (`*const T`, `*mut T`)
- **Problem:** Raw pointers do not implement `Send` or `Sync`.
- **Correction:** Use safe smart pointers (`Arc<T>`, `Box<T>`).

### 4. Non-`'static` References (`&'a T`)
- **Problem:** A struct holding a temporary reference (`&'a str` or `&'a Database`) cannot outlive the function that created it.
- **Compiler Error:** `lifetime ''static' required`
- **Correction:** Store owned types (`String`, `Vec<u8>`) or static string slices (`&'static str`).

---

## Concurrency Rules and Runtime Anti-Patterns

While Rust's compiler guarantees memory safety, performance and liveness require adherence to three operational rules:

### Rule 1: Never hold `std::sync::Mutex` across `.await`

Standard library mutex guards (`std::sync::MutexGuard`) do not implement `Send`. Holding a standard lock across an `.await` boundary prevents Tokio from moving the task between worker threads and will block the OS worker thread:

```rust
// INCORRECT: Causes compiler error or worker starvation
let mut lock = req.state.std_mutex.lock().unwrap();
let external_data = call_external_api().await; // AWAIT WITH LOCK HELD!
lock.push(external_data);
```

**Correct Approaches:**

*Approach A: Synchronous scope before or after await*
```rust
let external_data = call_external_api().await;
{
    let mut lock = req.state.std_mutex.lock().unwrap();
    lock.push(external_data);
} // Lock dropped immediately
```

*Approach B: Asynchronous Mutex*
```rust
// Use tokio::sync::Mutex if the lock MUST span across an await call
let mut lock = req.state.tokio_mutex.lock().await;
let external_data = call_external_api().await;
lock.push(external_data);
```

---

### Rule 2: Always pool external connections

Never attempt to share a single, un-pooled database connection handle across concurrent route handlers:

```rust
// INCORRECT: Fails `Sync` requirement and cannot handle concurrent queries
struct AppState {
    conn: rusqlite::Connection, // Not Sync
}
```

**Correct Approach:**
Always use a thread-safe connection pool (`sqlx::Pool`, `r2d2::Pool`, `deadpool::Pool`):

```rust
struct AppState {
    db: sqlx::PgPool, // Thread-safe, implements Send + Sync + 'static
}
```

---

### Rule 3: Offload CPU-heavy tasks to `spawn_blocking`

Async route handlers run on Tokio worker threads. Long-running synchronous CPU calculations or blocking filesystem calls stall other requests:

```rust
// INCORRECT: Freezes the worker thread for 200ms
app.get("/compute", |_req, res| async move {
    let result = heavy_cryptographic_work();
    res.json(result)
});
```

**Correct Approach:**
```rust
app.get("/compute", |_req, res| async move {
    let result = tokio::task::spawn_blocking(|| {
        heavy_cryptographic_work()
    }).await.map_err(|e| Error::Internal(e.to_string()))?;

    res.json(result)
});
```

---

## Modular Routers and Sub-Routers

When dividing routes across multiple modules or files, use `Router::<S>::with_state()`:

```rust
// routes/users.rs
use nitrox::Router;
use crate::AppState;

pub fn router() -> Router<AppState> {
    let mut r = Router::with_state();

    r.get("/", |req, res| async move {
        let users = req.state.user_service.get_all().await;
        res.json(users)
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

## Production Reference Example

For a complete, multi-file enterprise architecture demonstrating modular routers, controllers, services, configuration, and state injection, see [`examples/modular_api`](../examples/modular_api).

