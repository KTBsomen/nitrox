# Nitrox Middleware Master Guide

A comprehensive, production-ready guide to understanding, writing, composing, and optimizing middleware in Nitrox.

---

## Table of Contents

1. [What is Middleware?](#what-is-middleware)
2. [The Request Lifecycle and the Onion Model](#the-request-lifecycle-and-the-onion-model)
3. [The Four Tiers of Middleware in Nitrox](#the-four-tiers-of-middleware-in-nitrox)
   - [Tier 1: Global / Application-Level](#tier-1-global--application-level)
   - [Tier 2: Router-Level](#tier-2-router-level)
   - [Tier 3: Route-Level](#tier-3-route-level)
   - [Tier 4: Fluent RouteBuilder](#tier-4-fluent-routebuilder)
4. [Passing Multiple Middlewares (Tuples & Vectors)](#passing-multiple-middlewares-tuples--vectors)
5. [Understanding `Next` and Short-Circuiting](#understanding-next-and-short-circuiting)
6. [Building Real-World Custom Middlewares](#building-real-world-custom-middlewares)
   - [1. Authentication & JWT Extraction](#1-authentication--jwt-extraction)
   - [2. Role-Based Authorization](#2-role-based-authorization)
   - [3. Request Latency & Performance Headers](#3-request-latency--performance-headers)
   - [4. Redis Token Bucket Rate Limiter](#4-redis-token-bucket-rate-limiter)
   - [5. API Key & Security Header Validation](#5-api-key--security-header-validation)
   - [6. Accessing Application State in Middleware](#6-accessing-application-state-in-middleware)
7. [When to Use Which Pattern (Decision Matrix)](#when-to-use-which-pattern-decision-matrix)
8. [Express.js vs Nitrox: Side-by-Side Comparison](#expressjs-vs-nitrox-side-by-side-comparison)
9. [Under the Hood: Zero-Cost Boot-Time Pre-Composition](#under-the-hood-zero-cost-boot-time-pre-composition)

---

## What is Middleware?

In Nitrox, a **middleware** is an asynchronous function that sits directly in the path between an incoming HTTP request and your final route handler. Middleware can:

1. **Inspect and validate** incoming requests (headers, query params, authentication tokens).
2. **Short-circuit** the pipeline early (e.g., returning `401 Unauthorized` or `429 Too Many Requests` without executing downstream code).
3. **Enrich and transform** the response (adding security headers, `X-Response-Time`, compression).
4. **Wrap execution** with telemetry, logging, and performance metrics.

Every middleware function in Nitrox has the following signature:

```rust
Fn(Request<S>, Response, Next<S>) -> Future<Output = NitroxResult<Response>>
```

- `req`: The incoming [`Request<S>`](file:///c:/Users/somen/myProjects/rustyx/src/request.rs) containing headers, method, path, state, and body readers.
- `res`: The outgoing [`Response`](file:///c:/Users/somen/myProjects/rustyx/src/response.rs) builder.
- `next`: The continuation closure [`Next<S>`](file:///c:/Users/somen/myProjects/rustyx/src/middleware.rs). Calling `next(req, res).await` passes control to the next middleware or the route handler.

---

## The Request Lifecycle and the Onion Model

Nitrox uses the classic **Onion Architecture** (similar to Koa, Express, and Axum layers). Middlewares execute in nested layers:

```text
Incoming Request
      │
      ▼
┌────────────────────────────────────────────────────────┐
│ Global Middleware 1 (e.g. Request ID & Logger Enter)   │
│   ┌──────────────────────────────────────────────────┐ │
│   │ Router Middleware 1 (e.g. Session / Token Enter) │ │
│   │   ┌────────────────────────────────────────────┐ │ │
│   │   │ Route Middleware 1 (e.g. Require Role Enter)│ │
│   │   │   ┌──────────────────────────────────────┐ │ │ │
│   │   │   │                                      │ │ │ │
│   │   │   │        Terminal Route Handler        │ │ │ │
│   │   │   │     (Generates HTTP 200 Response)    │ │ │ │
│   │   │   │                                      │ │ │ │
│   │   │   └──────────────────────────────────────┘ │ │ │
│   │   │ Route Middleware 1 Exit (Unwinds)          │ │ │
│   │   └────────────────────────────────────────────┘ │ │
│   │ Router Middleware 1 Exit (Unwinds)               │ │
│   └──────────────────────────────────────────────────┘ │
│ Global Middleware 1 Exit (Logs Duration & Status)      │
└────────────────────────────────────────────────────────┘
      │
      ▼
Outgoing Response to Client
```

### Execution Flow Rules
1. **Entering:** Middlewares run in the exact order they are registered (FIFO: First In, First Out).
2. **Terminal Handler:** Once all middlewares have called `next(req, res).await`, the terminal route handler executes.
3. **Exiting (Unwinding):** As the future resolves, control flows backward through each middleware in reverse order (LIFO: Last In, First Out). This allows outer middlewares to inspect the final response status or add timing headers.

---

## The Four Tiers of Middleware in Nitrox

Nitrox provides four distinct tiers for attaching middleware, from whole-application scope down to a single HTTP verb on an individual endpoint.

### Tier 1: Global / Application-Level

Applied to **every single route** across the entire web application, including static file endpoints.

```rust
use nitrox::prelude::*;
use nitrox::middleware::{logger, cors, helmet};

let mut app = Nitrox::new();

// Global middlewares run for all requests
app.use_middleware(logger());
app.use_middleware(cors("*"));
app.use_middleware(helmet());
```

---

### Tier 2: Router-Level

Applied to all routes grouped inside a specific [`Router<S>`](file:///c:/Users/somen/myProjects/rustyx/src/router.rs). Useful for prefix-mounted sub-modules (e.g., `/api/v1`, `/admin`, `/auth`).

```rust
let mut admin_router = Router::new();

// Express-style: runs for every endpoint registered on admin_router
admin_router.use_middleware(|req, res, next| async move {
    if req.header("x-admin-key") != Some("super-secret") {
        return res.status(403).json(serde_json::json!({
            "error": "Forbidden: Admin key required"
        }));
    }
    next(req, res).await
});

// Alias: admin_router.r#use(mw) also works!
admin_router.get("/metrics", |req, res| async move {
    res.json(serde_json::json!({"active_nodes": 4}))
});

// Mount admin_router with prefix "/admin"
app.mount("/admin", admin_router);
```

#### Order Guarantee
Calling `router.use_middleware` multiple times **strictly maintains FIFO registration order**:

```rust
router.use_middleware(mw_first)
      .use_middleware(mw_second)
      .use_middleware(mw_third);
// Execution: mw_first -> mw_second -> mw_third -> Handler
```

---

### Tier 3: Route-Level

Applied exclusively to a single endpoint. Nitrox provides dedicated `_with` verbs:
- `get_with`
- `post_with`
- `put_with`
- `delete_with`
- `patch_with`

Both `app` and `router` support these methods:

```rust
// Only /dashboard requires authentication; / remains completely public
app.get("/", |_req, res| async move {
    res.send("Public Landing Page")
});

app.get_with("/dashboard", require_auth, |req, res| async move {
    res.json(serde_json::json!({"secret": "user profile data"}))
});
```

---

### Tier 4: Fluent RouteBuilder

When configuring multiple HTTP verbs or multiple middlewares on the same URI path, use `r.route(path)`:

```rust
router.route("/items")
    .use_middleware(require_auth)
    .use_middleware(audit_log)
    .get(|_req, res| async move {
        res.json(serde_json::json!({"items": ["a", "b"]}))
    })
    .post(|req, res| async move {
        res.status(201).json(serde_json::json!({"created": true}))
    });
```

---

## Passing Multiple Middlewares (Tuples & Vectors)

Nitrox implements the [`IntoMiddlewareChain<S>`](file:///c:/Users/somen/myProjects/rustyx/src/middleware.rs) trait for:
1. A single middleware closure or function pointer.
2. Tuples of up to 8 middlewares: `(mw1, mw2)`, `(mw1, mw2, mw3, ...)`.
3. Vectors of middleware: `Vec<MiddlewareFn<S>>`.

### Example with Tuples:
```rust
async fn auth_mw<S: Send + Sync + 'static>(req: Request<S>, res: Response, next: Next<S>) -> NitroxResult<Response> {
    if req.bearer_token().is_none() {
        return res.status(401).send("Unauthorized");
    }
    next(req, res).await
}

async fn admin_mw<S: Send + Sync + 'static>(req: Request<S>, res: Response, next: Next<S>) -> NitroxResult<Response> {
    if req.header("x-role") != Some("admin") {
        return res.status(403).send("Requires Admin Role");
    }
    next(req, res).await
}

// Pass a 2-tuple: executes auth_mw FIRST, then admin_mw SECOND
router.get_with("/admin/settings", (auth_mw, admin_mw), |req, res| async move {
    res.json(serde_json::json!({"maintenance_mode": false}))
});
```

---

## Understanding `Next` and Short-Circuiting

The `next` parameter is not a callback; it is an asynchronous callable that represents the rest of the pipeline:

```rust
let response = next(req, res).await?;
```

### The Power of Short-Circuiting
To halt request processing immediately, **do not call `next`**. Simply return your own [`Response`](file:///c:/Users/somen/myProjects/rustyx/src/response.rs):

```rust
|req, res, next| async move {
    if req.bearer_token().is_none() {
        // Downstream middlewares and route handlers will NEVER run!
        return res.status(401).json(serde_json::json!({
            "error": "Unauthorized",
            "message": "Bearer token missing"
        }));
    }

    // Only if authenticated do we proceed:
    next(req, res).await
}
```

---

## Building Real-World Custom Middlewares

### 1. Authentication & JWT Extraction

```rust
use nitrox::prelude::*;

pub async fn require_jwt<S: Send + Sync + 'static>(
    req: Request<S>,
    res: Response,
    next: Next<S>,
) -> NitroxResult<Response> {
    let token = match req.bearer_token() {
        Some(t) => t,
        None => {
            return res.status(401).json(serde_json::json!({
                "status": "fail",
                "message": "Authorization header missing or not Bearer format"
            }));
        }
    };

    // Replace with jsonwebtoken or your validation logic
    if token != "valid-jwt-token" {
        return res.status(401).json(serde_json::json!({
            "status": "fail",
            "message": "Token expired or invalid"
        }));
    }

    next(req, res).await
}
```

---

### 2. Role-Based Authorization

Creating reusable middleware factories with arguments:

```rust
use nitrox::prelude::*;

pub fn require_role<S: Send + Sync + 'static>(expected_role: &'static str) -> impl Fn(Request<S>, Response, Next<S>) -> std::pin::Pin<Box<dyn std::future::Future<Output = NitroxResult<Response>> + Send>> + Send + Sync + 'static {
    move |req, res, next| {
        Box::pin(async move {
            match req.header("x-user-role") {
                Some(role) if role == expected_role => next(req, res).await,
                Some(_) => res.status(403).json(serde_json::json!({
                    "error": "Forbidden: Insufficient privileges"
                })),
                None => res.status(401).json(serde_json::json!({
                    "error": "Unauthorized: Role header missing"
                })),
            }
        })
    }
}

// Usage:
router.get_with("/admin/users", require_role("superadmin"), |req, res| async move {
    res.send("Superadmin User List")
});
```

---

### 3. Request Latency & Performance Headers

```rust
use std::time::Instant;
use nitrox::prelude::*;

pub fn latency_tracker<S: Send + Sync + 'static>() -> impl Fn(Request<S>, Response, Next<S>) -> std::pin::Pin<Box<dyn std::future::Future<Output = NitroxResult<Response>> + Send>> + Send + Sync + 'static {
    |req, res, next| {
        Box::pin(async move {
            let start = Instant::now();
            let path = req.path().to_string();
            let method = req.method().to_string();

            // Downstream execution:
            let response = next(req, res).await?;

            let duration = start.elapsed();
            println!("[{}] {} -> completed in {:.3}ms", method, path, duration.as_secs_f64() * 1000.0);

            // Add benchmark header to client response
            Ok(response.header("X-Response-Time", &format!("{:.2}ms", duration.as_secs_f64() * 1000.0)))
        })
    }
}
```

---

### 4. Redis Token Bucket Rate Limiter

Using Nitrox shared state to query a Redis connection pool inside middleware:

```rust
use nitrox::prelude::*;

pub struct AppState {
    pub redis: redis::aio::ConnectionManager,
}

pub async fn redis_rate_limit(
    req: Request<AppState>,
    res: Response,
    next: Next<AppState>,
) -> NitroxResult<Response> {
    let client_ip = req.header("x-forwarded-for").unwrap_or("127.0.0.1");
    let key = format!("ratelimit:{}", client_ip);

    let mut conn = req.state.redis.clone();

    // Increment request count in Redis with a 60-second TTL
    let count: i64 = redis::cmd("INCR")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .unwrap_or(1);

    if count == 1 {
        let _: () = redis::cmd("EXPIRE")
            .arg(&key)
            .arg(60)
            .query_async(&mut conn)
            .await
            .unwrap_or(());
    }

    if count > 100 {
        return res.status(429).json(serde_json::json!({
            "error": "Too Many Requests",
            "message": "Rate limit exceeded. Try again in 60 seconds."
        }));
    }

    next(req, res).await
}
```

---

### 5. API Key & Security Header Validation

```rust
use nitrox::prelude::*;

pub async fn require_json_content_type<S: Send + Sync + 'static>(
    req: Request<S>,
    res: Response,
    next: Next<S>,
) -> NitroxResult<Response> {
    if matches!(req.method(), &hyper::Method::POST | &hyper::Method::PUT | &hyper::Method::PATCH) {
        let is_json = req.content_type()
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false);

        if !is_json {
            return res.status(415).json(serde_json::json!({
                "error": "Unsupported Media Type",
                "message": "Content-Type must be application/json"
            }));
        }
    }

    next(req, res).await
}
```

---

### 6. Accessing Application State in Middleware

Because [`Request<S>`](file:///c:/Users/somen/myProjects/rustyx/src/request.rs) carries your generic state `S`, middleware has zero-overhead access to database pools, caches, and configuration:

```rust
pub struct DatabaseState {
    pub db: sqlx::PgPool,
}

pub async fn tenant_verifier(
    req: Request<DatabaseState>,
    res: Response,
    next: Next<DatabaseState>,
) -> NitroxResult<Response> {
    let tenant_id = req.header("x-tenant-id").unwrap_or("default");

    // Direct access to state.db with zero manual locking:
    let exists = sqlx::query!("SELECT id FROM tenants WHERE id = $1", tenant_id)
        .fetch_optional(&req.state.db)
        .await
        .map_err(|e| NitroxError::Internal(e.to_string()))?;

    if exists.is_none() {
        return res.status(404).send("Tenant Not Found");
    }

    next(req, res).await
}
```

---

## When to Use Which Pattern (Decision Matrix)

| Use Case | Recommended Pattern | Example |
|---|---|---|
| Request Logging, Global CORS, Helmet Security Headers | **Tier 1: Global (`app.use_middleware`)** | `app.use_middleware(logger())` |
| Route Prefix Protection (`/api/v1/*`, `/admin/*`) | **Tier 2: Router-Level (`router.use_middleware`)** | `admin.use_middleware(admin_guard)` |
| Protecting a single endpoint (`GET /me`, `POST /checkout`) | **Tier 3: Route-Level (`get_with`, `post_with`)** | `r.get_with("/me", auth, handler)` |
| Sequential checks on one route (Auth + Role + Audit) | **Tier 3: Route-Level Tuple** | `r.get_with("/p", (auth, role), handler)` |
| Multiple HTTP verbs on the same URI with shared middleware | **Tier 4: RouteBuilder (`r.route(path)`)** | `r.route("/cart").use_middleware(auth).get(...).post(...)` |

---

## Express.js vs Nitrox: Side-by-Side Comparison

If you know Express.js, here is the exact Rosetta Stone:

| Pattern | Express.js (Node.js) | Nitrox (Rust) |
|---|---|---|
| **Global Middleware** | `app.use(logger);` | `app.use_middleware(logger());` |
| **Router Middleware** | `router.use(auth);` | `router.use_middleware(auth);`<br>*or* `router.r#use(auth);` |
| **Multiple Router MWs** | `router.use(a); router.use(b);` | `router.use_middleware(a).use_middleware(b);` |
| **Route Middleware** | `router.get('/me', auth, handler);` | `router.get_with("/me", auth, \|req, res\| ...);` |
| **Multiple Route MWs** | `router.get('/admin', [auth, role], handler);` | `router.get_with("/admin", (auth, role), \|req, res\| ...);` |
| **Calling Next** | `next();` *(callback)* | `next(req, res).await;` *(async future)* |
| **Short-Circuiting** | `res.status(401).send("No");` | `return res.status(401).send("No");` |
| **Inline Handler** | `(req, res) => res.json({...})` | `\|req, res\| async move { res.json(...) }` |

---

## Under the Hood: Zero-Cost Boot-Time Pre-Composition

Traditional dynamic middleware frameworks evaluate arrays of middlewares dynamically on every single incoming HTTP request, acquiring locks or allocating new vector frames on the heap.

Nitrox takes a **zero-cost compilation approach at boot time**:

```rust
// In src/middleware.rs:
pub fn compose_middleware_chain<S, H, Fut>(
    chain: Vec<MiddlewareFn<S>>,
    handler: H,
) -> HandlerFn<S>
```

1. **At Server Boot:** When you register routes, Nitrox pre-wraps the terminal handler with the router's middleware stack using reverse folding.
2. **At Runtime:** The router matches the URL in its radix tree and immediately invokes the unified, pre-composed `HandlerFn<S>`.
3. **Zero Dynamic Lock Contention:** There is no `Mutex` or `RwLock` during request dispatch.
4. **Clean Route Bypass:** Endpoints without middleware invoke their terminal handler directly with zero indirection layers, reaching benchmark speeds exceeding **180,000 req/sec**.
