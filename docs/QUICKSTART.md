# Quick Start Guide

Get up and running with Nitrox in 5 minutes!

## Hello World

```rust
use nitrox::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Nitrox::new();

    app.get("/", |_req, res| async move {
        res.send("Hello, World!")
    });

    app.listen(3000).await
}
```

Run with `cargo run` and visit `http://localhost:3000`.

---

## JSON API

```rust
use nitrox::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Nitrox::new();

    app.get("/api/status", |_req, res| async move {
        res.json(json!({
            "status": "ok",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    });

    app.listen(3000).await
}
```

---

## URL Parameters

```rust
// Route: /users/:id
app.get("/users/:id", |req, res| async move {
    let id = req.param("id").unwrap();
    res.json(json!({ "user_id": id }))
});

// Route: /users/:userId/posts/:postId
app.get("/users/:userId/posts/:postId", |req, res| async move {
    let user_id = req.param("userId").unwrap();
    let post_id = req.param("postId").unwrap();
    res.json(json!({
        "user": user_id,
        "post": post_id
    }))
});
```

---

## Query Parameters

```rust
// URL: /search?q=rust&page=1&limit=10
app.get("/search", |req, res| async move {
    let query = req.query_param("q").cloned().unwrap_or_default();
    let page: u32 = req.query_param("page")
        .and_then(|p| p.parse().ok())
        .unwrap_or(1);
    let limit: u32 = req.query_param("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(10);

    res.json(json!({
        "search": query,
        "page": page,
        "limit": limit
    }))
});
```

---

## Handling POST Requests

```rust
#[derive(Debug, Serialize, Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    email: String,
}

app.post("/users", |req, res| async move {
    // Parse JSON body
    match req.json::<CreateUser>() {
        Ok(data) => {
            // Create user
            let user = User {
                id: uuid::Uuid::new_v4().to_string(),
                name: data.name,
                email: data.email,
            };
            
            // Return 201 Created
            res.status(201).json(json!({
                "message": "User created",
                "user": user
            }))
        }
        Err(e) => {
            // Return 400 Bad Request
            res.bad_request(&format!("Invalid JSON: {}", e))
        }
    }
});
```

---

## Zero-Clone Application State CRUD Example

Nitrox eliminates the traditional Rust closure-cloning nightmare. Using `Nitrox::with_state(state)`, your state is automatically wrapped in an internal atomic reference and directly available on `req.state` with **zero manual Arc wrapping and zero closure cloning**:

```rust
use nitrox::prelude::*;
use std::collections::HashMap;
use parking_lot::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: String,
    title: String,
    completed: bool,
}

// 1. Define your application state struct (No Arc wrapper needed!)
struct AppState {
    todos: RwLock<HashMap<String, Todo>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState {
        todos: RwLock::new(HashMap::new()),
    };

    // 2. Pass state directly to Nitrox - Nitrox handles Arc internally!
    let app = Nitrox::with_state(state);

    // List all todos - direct access via req.state!
    app.get("/todos", |req, res| async move {
        let todos: Vec<Todo> = req.state.todos.read().values().cloned().collect();
        res.json(json!({ "todos": todos }))
    });

    // Get single todo
    app.get("/todos/:id", |req, res| async move {
        let id = req.param("id").unwrap_or_default();
        let lock = req.state.todos.read();
        match lock.get(id) {
            Some(todo) => res.json(todo),
            None => res.not_found(),
        }
    });

    // Create todo
    app.post("/todos", |mut req, res| async move {
        #[derive(Deserialize)]
        struct CreateTodo { title: String }
        
        let data: CreateTodo = req.json().await?;
        let todo = Todo {
            id: uuid::Uuid::new_v4().to_string(),
            title: data.title,
            completed: false,
        };
        req.state.todos.write().insert(todo.id.clone(), todo.clone());
        res.status(201).json(todo)
    });

    // Delete todo
    app.delete("/todos/:id", |req, res| async move {
        let id = req.param("id").unwrap_or_default();
        match req.state.todos.write().remove(id) {
            Some(_) => res.no_content(),
            None => res.not_found(),
        }
    });

    app.listen(3000).await
}
```

---

## Adding Middleware

```rust
use nitrox::prelude::*;
use nitrox::middleware::*;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Nitrox::new();

    // Logging
    app.use_middleware(logger());
    
    // CORS
    app.use_middleware(cors("*"));
    
    // Security headers
    app.use_middleware(helmet());
    
    // Rate limiting (100 requests per minute)
    app.use_middleware(simple_rate_limit(100, 60));
    
    // Request timeout (30 seconds)
    app.use_middleware(timeout(30000));
    
    // Add request ID
    app.use_middleware(request_id());
    
    // Response time header
    app.use_middleware(response_time());

    app.get("/", |_req, res| async move {
        res.json(json!({ "message": "Protected API!" }))
    });

    app.listen(3000).await
}
```

---

## Error Handling

```rust
app.get("/risky", |_req, res| async move {
    // Using Result
    match some_risky_operation() {
        Ok(data) => res.json(data),
        Err(e) => res.internal_error(&e.to_string()),
    }
});

// Custom error handler for routes
app.get("/users/:id", |req, res| async move {
    let id = match req.param("id") {
        Some(id) => id,
        None => return res.bad_request("Missing user ID"),
    };

    // Validate ID format
    if id.len() != 36 {
        return res.bad_request("Invalid UUID format");
    }

    res.json(json!({ "id": id }))
});
```

---

## Using Routers

```rust
use nitrox::prelude::*;

fn user_routes() -> Router {
    let mut router = Router::new();
    
    router.get("/", |_req, res| async move {
        res.json(json!({ "users": [] }))
    });
    
    router.post("/", |req, res| async move {
        // Create user
        res.created(json!({ "id": "new-id" }))
    });
    
    router.get("/:id", |req, res| async move {
        let id = req.param("id").unwrap();
        res.json(json!({ "id": id }))
    });

    router
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = Nitrox::new();

    // Mount router at /api/users
    app.use_router("/api/users", user_routes());

    app.listen(3000).await
}
```

---

## Next Steps

- [Beginner's Guide & Express Migration](./BEGINNER_GUIDE.md) - Complete tutorial and Rust gotchas
- [Application State & Databases](./STATE.md) - Deep dive into state patterns and avoiding lock contention
- [Middleware Master Guide](./MIDDLEWARE.md) - 4-tier middleware lifecycle and recipes
- [Multipart File Uploads](./UPLOAD.md) - Multer-style file streaming
- [Deployment Guide](./DEPLOYMENT.md) - Deploy your application
