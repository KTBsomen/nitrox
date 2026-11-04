# 🔌 Middleware Guide

Middleware functions are the backbone of request processing in RustyX. They can modify requests, responses, and control the flow of request handling.

## How Middleware Works

```
Request → Middleware 1 → Middleware 2 → Route Handler → Response
              ↓               ↓               ↓
         (can modify)   (can modify)    (generates)
```

Each middleware receives:
- `Request` - The incoming request
- `Response` - The response builder
- `Next` - Function to call the next middleware/handler

---

## Built-in Middleware

### Logger

Logs all incoming requests with method, path, status, and duration.

```rust
use rustyx::middleware::logger;

app.use_middleware(logger());

// Output: GET /api/users 200 - 5.123ms
```

### CORS

Adds Cross-Origin Resource Sharing headers.

```rust
use rustyx::middleware::{cors, cors_with_options, CorsOptions};

// Simple - allow all origins
app.use_middleware(cors("*"));

// Allow specific origin
app.use_middleware(cors("https://example.com"));

// Advanced options
let options = CorsOptions::new()
    .origin("https://example.com")
    .credentials(true);
app.use_middleware(cors_with_options(options));
```

### Helmet

Adds security headers to protect against common vulnerabilities.

```rust
use rustyx::middleware::helmet;

app.use_middleware(helmet());

// Adds headers:
// - X-Content-Type-Options: nosniff
// - X-Frame-Options: DENY
// - X-XSS-Protection: 1; mode=block
// - Strict-Transport-Security: max-age=31536000; includeSubDomains
// - Content-Security-Policy: default-src 'self'
// - X-Permitted-Cross-Domain-Policies: none
// - Referrer-Policy: strict-origin-when-cross-origin
```

### Rate Limiting

Protect your API from abuse.

```rust
use rustyx::middleware::{rate_limiter, simple_rate_limit, RateLimiterConfig};

// Simple: 100 requests per 60 seconds
app.use_middleware(simple_rate_limit(100, 60));

// Advanced configuration
let config = RateLimiterConfig::new(100, 60)
    .message("Too many requests. Please slow down.")
    .skip(vec!["/health", "/metrics"]);

app.use_middleware(rate_limiter(config));
```

Response when rate limited:
```json
{
  "error": "Too Many Requests",
  "message": "Too many requests. Please slow down.",
  "retry_after": 45
}
```

Headers added:
- `X-RateLimit-Limit`: Maximum requests allowed
- `X-RateLimit-Remaining`: Requests remaining
- `Retry-After`: Seconds until rate limit resets (when exceeded)

### Timeout

Set a maximum time for request processing.

```rust
use rustyx::middleware::timeout;

// 30 second timeout
app.use_middleware(timeout(30000));

// Response on timeout:
// Status: 408 Request Timeout
// Body: {"error": "Request Timeout"}
```

### Request ID

Adds a unique ID to each request.

```rust
use rustyx::middleware::request_id;

app.use_middleware(request_id());

// Adds header: X-Request-ID: <uuid>
```

### Response Time

Adds response processing time header.

```rust
use rustyx::middleware::response_time;

app.use_middleware(response_time());

// Adds header: X-Response-Time: 5ms
```

---

## Custom Middleware

### Basic Custom Middleware

```rust
app.use_middleware(|req, res, next| async move {
    println!("Request: {} {}", req.method(), req.path());
    
    // Call next middleware/handler
    let response = next(req, res).await;
    
    println!("Response: {}", response.get_status());
    response
});
```

### Authentication Middleware

```rust
app.use_middleware(|req, res, next| async move {
    // Skip auth for public routes
    let public_paths = ["/", "/health", "/login", "/register"];
    if public_paths.contains(&req.path()) {
        return next(req, res).await;
    }

    // Check for Bearer token
    match req.bearer_token() {
        Some(token) => {
            // Validate token (implement your logic)
            if is_valid_token(token) {
                next(req, res).await
            } else {
                res.status(401).json(json!({
                    "error": "Unauthorized",
                    "message": "Invalid token"
                }))
            }
        }
        None => {
            res.status(401).json(json!({
                "error": "Unauthorized",
                "message": "No token provided"
            }))
        }
    }
});

fn is_valid_token(token: &str) -> bool {
    // Your JWT validation logic here
    !token.is_empty()
}
```

### API Key Middleware

```rust
app.use_middleware(|req, res, next| async move {
    let api_key = req.header("x-api-key");
    
    match api_key {
        Some(key) if key == "your-secret-key" => {
            next(req, res).await
        }
        _ => {
            res.status(403).json(json!({
                "error": "Forbidden",
                "message": "Invalid API key"
            }))
        }
    }
});
```

### Request Validation Middleware

```rust
app.use_middleware(|req, res, next| async move {
    // Require Content-Type for POST/PUT/PATCH
    let needs_content_type = matches!(
        req.method(),
        &hyper::Method::POST | &hyper::Method::PUT | &hyper::Method::PATCH
    );

    if needs_content_type {
        match req.content_type() {
            Some(ct) if ct.contains("application/json") => {
                next(req, res).await
            }
            _ => {
                res.status(415).json(json!({
                    "error": "Unsupported Media Type",
                    "message": "Content-Type must be application/json"
                }))
            }
        }
    } else {
        next(req, res).await
    }
});
```

### Compression Middleware

```rust
use flate2::write::GzEncoder;
use flate2::Compression;

app.use_middleware(|req, res, next| async move {
    // Check if client accepts gzip
    let accepts_gzip = req
        .header("accept-encoding")
        .map(|h| h.contains("gzip"))
        .unwrap_or(false);

    let response = next(req, res).await;

    if accepts_gzip {
        // Add Content-Encoding header
        response.header("content-encoding", "gzip")
    } else {
        response
    }
});
```

---

## Middleware Order

Middleware executes in the order it's added. Order matters!

```rust
// ✅ Correct order
app.use_middleware(request_id());      // Add request ID first
app.use_middleware(logger());          // Then log (includes request ID)
app.use_middleware(cors("*"));         // CORS before auth
app.use_middleware(auth_middleware()); // Auth before rate limiting
app.use_middleware(rate_limiter(...)); // Rate limit authenticated users

// ❌ Wrong order
app.use_middleware(rate_limiter(...)); // Rate limiting won't know the user
app.use_middleware(auth_middleware()); // Auth happens too late
```

### Recommended Order

1. **Request ID** - Tag request for tracing
2. **Response Time** - Start timing
3. **Logger** - Log request
4. **CORS** - Handle preflight
5. **Helmet** - Security headers
6. **Authentication** - Verify user
7. **Rate Limiting** - Protect API
8. **Validation** - Validate request
9. **Timeout** - Prevent hanging

---

## Conditional Middleware

### Skip Paths

```rust
app.use_middleware(|req, res, next| async move {
    let skip_paths = ["/health", "/metrics", "/public"];
    
    if skip_paths.iter().any(|p| req.path().starts_with(p)) {
        return next(req, res).await;
    }

    // Your middleware logic
    next(req, res).await
});
```

### Method-specific

```rust
app.use_middleware(|req, res, next| async move {
    // Only apply to mutation methods
    if !matches!(
        req.method(),
        &hyper::Method::POST | &hyper::Method::PUT | &hyper::Method::DELETE
    ) {
        return next(req, res).await;
    }

    // Check CSRF token
    match req.header("x-csrf-token") {
        Some(_) => next(req, res).await,
        None => res.forbidden(),
    }
});
```

---

## Next Steps

- [API Reference](./API.md) - Complete API documentation
- [Database Guide](./DATABASE.md) - Database integration
- [Deployment Guide](./DEPLOYMENT.md) - Production deployment
