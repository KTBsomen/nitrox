# Nitrox Beginner's Guide: From Express to High-Performance Rust

A complete, tutorial-style guide designed for JavaScript, TypeScript, Go, and beginner Rust developers. This document teaches you how to build production-grade web services in Nitrox, explaining every function in plain language and unpacking every Rust pitfall before it trips you up.

---

## Table of Contents

1. [The Mental Model: Express.js vs Nitrox](#the-mental-model-expressjs-vs-nitrox)
2. [Quick Start: Hello World in 3 Minutes](#quick-start-hello-world-in-3-minutes)
3. [Complete Developer Reference: Every Function Explained](#complete-developer-reference-every-function-explained)
   - [The Application (`Nitrox<S>`)](#the-application-nitroxs)
   - [The Router (`Router<S>`)](#the-router-routers)
   - [The Fluent RouteBuilder (`RouteBuilder<S>`)](#the-fluent-routebuilder-routebuilders)
   - [The Request Object (`Request<S>`)](#the-request-object-requests)
   - [The Response Builder (`Response`)](#the-response-builder-response)
4. [Rust Gotchas for Beginners (Save 40 Hours of Debugging)](#rust-gotchas-for-beginners-save-40-hours-of-debugging)
   - [Gotcha 1: The Deadly `unwrap()` Trap in Production](#gotcha-1-the-deadly-unwrap-trap-in-production)
   - [Gotcha 2: `Option` (`Some`/`None`) vs `Result` (`Ok`/`Err`)](#gotcha-2-option-somenone-vs-result-okerr)
   - [Gotcha 3: The Modern `let else` Superpower](#gotcha-3-the-modern-let-else-superpower)
   - [Gotcha 4: When and How to Use the `?` Operator](#gotcha-4-when-and-how-to-use-the--operator)
   - [Gotcha 5: JSON Parsing and the Fatal `.to_string()` Gotcha](#gotcha-5-json-parsing-and-the-fatal-to_string-gotcha)
   - [Gotcha 6: Database Connection Pools and the `Mutex` Trap](#gotcha-6-database-connection-pools-and-the-mutex-trap)
   - [Gotcha 7: The `async move` Closure Rule](#gotcha-7-the-async-move-closure-rule)
5. [End-to-End Production Project: Auth, Redis, MongoDB/PostgreSQL & Middlewares](#end-to-end-production-project-auth-redis-mongodbpostgresql--middlewares)

---

## The Mental Model: Express.js vs Nitrox

If you are coming from Node.js and Express:

| Concept | Express.js (Node.js) | Nitrox (Rust) | Why Nitrox is Faster & Safer |
|---|---|---|---|
| **Runtime** | Single-threaded Event Loop (V8) | Multi-threaded Work-Stealing Tokio Runtime | Utilizes 100% of CPU cores automatically |
| **Creation** | `const app = express();` | `let mut app = Nitrox::new();` | Zero runtime baggage, direct Hyper 1.x binding |
| **App State** | `app.locals.db = db;` *(untyped)* | `Nitrox::with_state(state)` *(strictly typed)* | Compiler verifies thread safety (`Send + Sync`) |
| **Route Params** | `req.params.id` | `req.param("id")` | Clean Option-based extraction with zero panic risk |
| **Query String** | `req.query.page` | `req.query("page")` | Built-in URL query decoding |
| **JSON Body** | `req.body` (needs `express.json()`) | `req.json::<T>().await?` | Zero-copy deserialization directly into Rust structs |
| **Middlewares** | `app.use(fn)` / `next()` callback | `app.use_middleware(fn)` / `next(req, res).await` | Async futures with compile-time type verification |
| **Error Handling**| `try { ... } catch (err)` | `Result<Response, NitroxError>` & `?` | No unhandled runtime exceptions or silent crashes |

---

## Quick Start: Hello World in 3 Minutes

### Step 1: Initialize your Rust project

```bash
cargo new my_nitrox_api
cd my_nitrox_api
```

### Step 2: Configure `Cargo.toml`

Open `Cargo.toml` and add:

```toml
[dependencies]
nitrox = "0.0.2"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Step 3: Write your server in `src/main.rs`

```rust
use nitrox::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Nitrox::new();

    // Standard health check
    app.get("/health", |_req, res| async move {
        res.json(serde_json::json!({
            "status": "ok",
            "uptime_seconds": 120
        }))
    });

    // Dynamic URL parameter: /hello/:name
    app.get("/hello/:name", |req, res| async move {
        let name = req.param("name").unwrap_or("World");
        res.send(format!("Hello, {}!", name))
    });

    println!("Server running on http://127.0.0.1:3000");
    app.listen(3000).await?;
    Ok(())
}
```

### Step 4: Run it!

```bash
cargo run
```

Test with curl:
```bash
curl http://127.0.0.1:3000/health
curl http://127.0.0.1:3000/hello/Rustacean
```

---

## Complete Developer Reference: Every Function Explained

### The Application (`Nitrox<S>`)

[`Nitrox<S>`](file:///c:/Users/somen/myProjects/rustyx/src/app.rs) is the primary server instance where you attach routes, configure shared state, mount sub-routers, and start listening.

| Method | What it does in plain language | Express Equivalent |
|---|---|---|
| `Nitrox::new()` | Creates a stateless server instance. | `express()` |
| `Nitrox::with_state(state)` | Creates a server with shared application state (database, Redis, configuration). | Storing items in `app.locals` |
| `app.get(path, handler)` | Registers a `GET` endpoint. | `app.get(path, handler)` |
| `app.post(path, handler)` | Registers a `POST` endpoint. | `app.post(path, handler)` |
| `app.put(path, handler)` | Registers a `PUT` endpoint. | `app.put(path, handler)` |
| `app.delete(path, handler)` | Registers a `DELETE` endpoint. | `app.delete(path, handler)` |
| `app.patch(path, handler)` | Registers a `PATCH` endpoint. | `app.patch(path, handler)` |
| `app.get_with(path, mw, handler)` | Registers a `GET` endpoint with route-specific middleware(s). | `app.get(path, mw, handler)` |
| `app.use_middleware(mw)` | Registers a global middleware executed on every request. | `app.use(mw)` |
| `app.mount(prefix, router)` | Mounts a sub-router with a common URL prefix (e.g., `/api/v1`). | `app.use('/api/v1', router)` |
| `app.static(url_path, dir)` | Serves static assets from a local disk folder. | `app.use('/public', express.static('public'))` |
| `app.listen(port).await` | Binds to TCP port and starts processing requests asynchronously. | `app.listen(port, () => ...)` |

---

### The Router (`Router<S>`)

[`Router<S>`](file:///c:/Users/somen/myProjects/rustyx/src/router.rs) lets you split large APIs into clean, isolated controller files.

| Method | What it does in plain language | Express Equivalent |
|---|---|---|
| `Router::new()` | Creates a new isolated sub-router. | `express.Router()` |
| `router.use_middleware(mw)` | Runs middleware for every route in this router. Strict FIFO order! | `router.use(mw)` |
| `router.r#use(mw)` | Express-friendly alias for `use_middleware`. | `router.use(mw)` |
| `router.get(path, handler)` | Adds a `GET` route to this router. | `router.get(path, handler)` |
| `router.get_with(path, mw, handler)` | Adds a `GET` route with specific middleware(s) (single, tuple, or vec). | `router.get(path, mw, handler)` |
| `router.route(path)` | Starts a fluent builder for multiple methods or middlewares on a path. | `router.route(path)` |

---

### The Fluent RouteBuilder (`RouteBuilder<S>`)

Chain multiple middlewares and methods on the exact same endpoint:

```rust
router.route("/users")
    .use_middleware(require_auth)
    .get(|req, res| async move {
        res.json(serde_json::json!({"users": []}))
    })
    .post(|req, res| async move {
        res.status(201).send("User created")
    });
```

---

### The Request Object (`Request<S>`)

Every incoming request provides clean helper methods:

| Method | Return Type | Description |
|---|---|---|
| `req.method()` | `&Method` | HTTP method (e.g., `Method::GET`, `Method::POST`). |
| `req.path()` | `&str` | Path of the request (e.g., `"/api/users"`). |
| `req.param("id")` | `Option<&str>` | Extracts URL path parameter `:id`. Returns `None` if not found. |
| `req.params()` | `&HashMap<String, String>` | Map containing all path parameters. |
| `req.query("search")` | `Option<&str>` | Extracts query string `?search=rust`. |
| `req.queries()` | `&HashMap<String, String>` | Map containing all parsed query parameters. |
| `req.header("authorization")` | `Option<&str>` | Case-insensitive lookup of request headers. |
| `req.bearer_token()` | `Option<&str>` | Parses `Authorization: Bearer <token>` automatically! |
| `req.content_type()` | `Option<&str>` | Returns the `Content-Type` header string. |
| `req.json::<T>().await?` | `NitroxResult<T>` | Deserializes the request body directly into struct `T`. |
| `req.body_string().await?` | `NitroxResult<String>` | Reads the request body as raw UTF-8 text. |
| `req.body_bytes().await?` | `NitroxResult<Bytes>` | Reads raw binary request payload (useful for files or binary data). |
| `req.state` *(or `req.state()`)* | `&S` | Direct zero-clone access to your shared application state! |

---

### The Response Builder (`Response`)

The response builder uses fluent method chaining:

| Method | Description | Example |
|---|---|---|
| `res.status(code)` | Sets the HTTP status code. | `res.status(201)` |
| `res.json(val)` | Serializes JSON and sets `Content-Type: application/json`. | `res.json(json!({"ok": true}))` |
| `res.send(text)` | Sends plain text body with status 200. | `res.send("Hello")` |
| `res.send_bytes(bytes)` | Sends raw binary bytes. | `res.send_bytes(vec![0x89, 0x50])` |
| `res.html(string)` | Sends HTML string with `Content-Type: text/html`. | `res.html("<h1>Home</h1>")` |
| `res.header(key, val)` | Adds a custom HTTP header. | `res.header("X-App-Version", "1.0")` |
| `res.cookie(name, val, opts)` | Sets a Set-Cookie header with options. | `res.cookie("token", "xyz", CookieOptions::new().http_only(true))` |
| `res.redirect(url)` | Sends HTTP 302 redirect. | `res.redirect("/login")` |
| `res.cors(origin)` | Adds standard CORS headers. | `res.cors("*")` |

---

## Rust Gotchas for Beginners (Save 40 Hours of Debugging)

### Gotcha 1: The Deadly `unwrap()` Trap in Production

In JavaScript/TypeScript, if you access a property that doesn't exist, you get `undefined`:
```javascript
const role = req.headers["x-role"]; // undefined if missing
```

In beginner Rust, people often write:
```rust
// DANGEROUS! If header is missing, your server thread PANICS!
let role = req.header("x-role").unwrap(); 
```

#### Why `.unwrap()` is Dangerous:
If a client sends a request without the header, `.unwrap()` panics immediately. While Tokio keeps the server alive, that individual request crashes abruptly and returns an empty socket drop or internal error.

#### Safe Solutions:
1. **Provide a fallback with `.unwrap_or()`:**
   ```rust
   let role = req.header("x-role").unwrap_or("guest");
   ```
2. **Compute expensive fallbacks with `.unwrap_or_else()`:**
   ```rust
   let session = req.header("x-session").unwrap_or_else(|| generate_new_session());
   ```
3. **Use default values with `.unwrap_or_default()`:**
   ```rust
   let count: u32 = req.query("count").and_then(|v| v.parse().ok()).unwrap_or_default(); // defaults to 0
   ```

---

### Gotcha 2: `Option` (`Some`/`None`) vs `Result` (`Ok`/`Err`)

Rust has **no `null` or `undefined`**. Everything is wrapped in either an `Option` or a `Result`:

- `Option<T>`: Represents existence (`Some(value)`) or absence (`None`).
  - Example: `req.header("key")` returns `Option<&str>`.
- `Result<T, E>`: Represents success (`Ok(value)`) or failure (`Err(error)`).
  - Example: `req.json::<User>().await` returns `Result<User, NitroxError>`.

---

### Gotcha 3: The Modern `let else` Superpower

When validating headers or parameters, beginners often write deeply nested `match` statements:

```rust
// The Nested Pyramid of Doom:
let user_id = match req.param("id") {
    Some(id) => id,
    None => return res.status(400).json(json!({"error": "Missing ID"})),
};
```

#### The Better Way: `let else` (Introduced in Rust 1.65+)
`let else` lets you match the happy path (`Some`) and immediately return or break on `None`:

```rust
// Clean, flat, production-grade:
let Some(user_id) = req.param("id") else {
    return res.status(400).json(serde_json::json!({
        "error": "Missing user ID parameter"
    }));
};

let Some(token) = req.bearer_token() else {
    return res.status(401).json(serde_json::json!({
        "error": "Bearer token required"
    }));
};

// Both user_id and token are unwrapped and ready to use below!
```

---

### Gotcha 4: When and How to Use the `?` Operator

The `?` operator is Rust's early-exit shortcut for `Result`:
```rust
let body: CreateUserDto = req.json().await?;
```
If `req.json().await` fails (for instance, the client sent invalid JSON syntax), the `?` operator immediately returns the `NitroxError` from your handler function. Nitrox automatically catches this and returns a clean `400 Bad Request` to the client.

#### When can you NOT use `?` directly?
When the error type does NOT match `NitroxError`. For example, a raw SQLx or MongoDB error:

```rust
// Fails to compile if Error types don't match:
let user = sqlx::query!("...").fetch_one(&db).await?; // Error: sqlx::Error != NitroxError
```

#### How to fix it:
Convert the error using `.map_err()`:

```rust
let user = sqlx::query!("SELECT * FROM users WHERE id = $1", user_id)
    .fetch_optional(&req.state.db)
    .await
    .map_err(|e| NitroxError::Internal(format!("Database query failed: {}", e)))?;

let Some(user) = user else {
    return res.status(404).json(serde_json::json!({"error": "User not found"}));
};
```

---

### Gotcha 5: JSON Parsing and the Fatal `.to_string()` Gotcha

This is the single most common mistake JavaScript developers make when switching to Rust with `serde_json`.

Suppose a client sends `{"username": "somen"}`.

#### The Trap:
```rust
let body: serde_json::Value = req.json().await?;

// You want the string "somen", so you write:
let username = body["username"].to_string(); 
```

**WHAT ACTUALLY HAPPENS:**
`body["username"].to_string()` serializes the JSON value back into JSON text.
It returns `"\"somen\""` with **literal double quotes included!**

If you search your database with `"\"somen\""`, it looks for a user whose literal name is `"somen"` with quotes. The query returns 0 rows, and developers spend hours wondering why their database queries fail!

#### The Fix:
Always use `.as_str()` when extracting strings from `serde_json::Value`:

```rust
// Correct: returns Option<&str> ("somen" without quotes)
let Some(username) = body["username"].as_str() else {
    return res.status(400).send("Field 'username' must be a valid string");
};
```

#### Comparison Table:
| Method on `Value::String("admin")` | Result | Quotes Included? | Safe for DB queries? |
|---|---|---|---|
| `.to_string()` | `String` (`"\"admin\""`) | **YES (literal quotes)** | **NO! Causes query bugs** |
| `.as_str()` | `Option<&str>` (`Some("admin")`) | **NO** | **YES (Recommended)** |
| Strongly Typed Struct | `String` (`"admin"`) | **NO** | **YES (Best Practice)** |

#### Best Practice: Use Strongly Typed Structs
Instead of untyped `serde_json::Value`, define a struct with `#[derive(Deserialize)]`:

```rust
#[derive(serde::Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
    age: Option<u32>, // optional field
}

app.post("/users", |req, res| async move {
    let payload: CreateUserRequest = req.json().await?;
    // payload.username is guaranteed to be clean String "somen" without quotes!
    res.status(201).json(serde_json::json!({ "created": payload.username }))
});
```

---

### Gotcha 6: Database Connection Pools and the `Mutex` Trap

In Node.js, you have a single thread, so passing around a database pool is simple.
In Rust, multiple threads handle incoming requests simultaneously.

#### The Fatal Beginner Trap:
Beginners see the compiler complain about sharing state across threads, so they wrap their entire database pool in a `Mutex`:

```rust
// DO NOT DO THIS!
struct AppState {
    db: Arc<tokio::sync::Mutex<sqlx::PgPool>>, // DISASTER FOR PERFORMANCE!
}
```

#### Why is this a disaster?
A connection pool (like `sqlx::PgPool` or `mongodb::Client`) is **already thread-safe and manages its own internal pool of connections**.
If you put a `Mutex` around it, **only one HTTP request can query the database at a time**! Every other concurrent user is forced to wait in a single file queue. Your 100,000 req/sec server drops to 200 req/sec!

#### The Correct Nitrox Architecture:
Connection pools are designed to be cheap to clone (they are internal atomic pointers). Store them directly in your state struct:

```rust
#[derive(Clone)]
pub struct AppState {
    pub pg_pool: sqlx::PgPool,                     // Cheap clone!
    pub mongo_db: mongodb::Database,               // Cheap clone!
    pub redis: redis::aio::ConnectionManager,      // Cheap clone!
}
```

Then initialize with:
```rust
let state = AppState { pg_pool, mongo_db, redis };
let app = Nitrox::with_state(state);
```
Inside any route or middleware, `req.state.pg_pool` is directly accessible with zero locks and maximum concurrency!

---

### Gotcha 7: The `async move` Closure Rule

In Rust, route handlers must be `async move`:
```rust
app.get("/data", |_req, res| async move {
    res.send("ok")
});
```
The `move` keyword transfers ownership of variables into the async task so Tokio can execute the future on another thread safely.

If you have a local variable outside the route that you need inside the handler, **clone it before the closure**:

```rust
let secret_key = "my-secret-key".to_string();

// Clone outside:
let key_clone = secret_key.clone();
app.get("/auth", move |_req, res| {
    let key = key_clone.clone();
    async move {
        res.send(format!("Key: {}", key))
    }
});
```
*(Even better: put shared variables in your `AppState`, and access them through `req.state`!)*

---

## End-to-End Production Project: Auth, Redis, MongoDB/PostgreSQL & Middlewares

Here is a complete, copy-pasteable real-world example demonstrating:
1. Shared `AppState` with PostgreSQL and Redis.
2. Route-level authentication middleware with `get_with`.
3. Router-level logging middleware.
4. Clean JSON parsing and error handling without dangerous unwraps.

```rust
use nitrox::prelude::*;
use serde::{Deserialize, Serialize};

// 1. Shared State with Database and Cache
#[derive(Clone)]
pub struct AppState {
    pub db_url: String,
    // In production, include:
    // pub pool: sqlx::PgPool,
    // pub redis: redis::aio::ConnectionManager,
}

// 2. Strongly Typed Request/Response Models
#[derive(Deserialize)]
pub struct CreateArticleRequest {
    pub title: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ArticleResponse {
    pub id: u64,
    pub title: String,
    pub author: String,
}

// 3. Reusable Authentication Middleware
pub async fn require_auth(
    req: Request<AppState>,
    res: Response,
    next: Next<AppState>,
) -> NitroxResult<Response> {
    let Some(token) = req.bearer_token() else {
        return res.status(401).json(serde_json::json!({
            "error": "Unauthorized",
            "message": "Authorization header with Bearer token is required"
        }));
    };

    // Check token (e.g. against Redis or JWT decoder)
    if token != "secret-token-123" {
        return res.status(401).json(serde_json::json!({
            "error": "Unauthorized",
            "message": "Invalid or expired session token"
        }));
    }

    next(req, res).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState {
        db_url: "postgres://user:pass@localhost:5432/my_app".to_string(),
    };

    let mut app = Nitrox::with_state(state);

    // Global Middlewares
    app.use_middleware(nitrox::middleware::logger());
    app.use_middleware(nitrox::middleware::helmet());

    // Public Route
    app.get("/health", |_req, res| async move {
        res.json(serde_json::json!({ "status": "healthy" }))
    });

    // 4. Create Modular Sub-Router for Articles
    let mut articles_router = Router::with_state();

    // Route-level middleware using get_with
    articles_router.get_with("/profile", require_auth, |req, res| async move {
        res.json(serde_json::json!({
            "username": "somen",
            "tier": "enterprise"
        }))
    });

    // POST /api/v1/articles (Protected with require_auth)
    articles_router.post_with("/", require_auth, |mut req, res| async move {
        // Safe typed parsing with ?
        let payload: CreateArticleRequest = req.json().await?;

        // Access shared state directly
        println!("Connecting to database at: {}", req.state.db_url);

        let created_article = ArticleResponse {
            id: 42,
            title: payload.title,
            author: "Authenticated User".to_string(),
        };

        res.status(201).json(created_article)
    });

    // Mount sub-router at /api/v1/articles
    app.mount("/api/v1/articles", articles_router);

    println!("Starting production server on http://127.0.0.1:3000");
    app.listen(3000).await?;

    Ok(())
}
```

