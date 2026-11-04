# 🚀 Quick Start Guide

Get up and running with RustyX in 5 minutes!

## Hello World

```rust
use rustyx::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let app = RustyX::new();

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
use rustyx::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let app = RustyX::new();

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

## CRUD Example

```rust
use rustyx::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: String,
    title: String,
    completed: bool,
}

type Db = Arc<RwLock<HashMap<String, Todo>>>;

#[tokio::main]
async fn main() -> Result<()> {
    let db: Db = Arc::new(RwLock::new(HashMap::new()));
    let app = RustyX::new();

    // List all todos
    let db_clone = db.clone();
    app.get("/todos", move |_req, res| {
        let db = db_clone.clone();
        async move {
            let todos: Vec<Todo> = db.read().values().cloned().collect();
            res.json(json!({ "todos": todos }))
        }
    });

    // Get single todo
    let db_clone = db.clone();
    app.get("/todos/:id", move |req, res| {
        let db = db_clone.clone();
        async move {
            let id = req.param("id").unwrap();
            match db.read().get(id) {
                Some(todo) => res.json(todo),
                None => res.not_found(),
            }
        }
    });

    // Create todo
    let db_clone = db.clone();
    app.post("/todos", move |req, res| {
        let db = db_clone.clone();
        async move {
            #[derive(Deserialize)]
            struct CreateTodo { title: String }
            
            match req.json::<CreateTodo>() {
                Ok(data) => {
                    let todo = Todo {
                        id: uuid::Uuid::new_v4().to_string(),
                        title: data.title,
                        completed: false,
                    };
                    db.write().insert(todo.id.clone(), todo.clone());
                    res.created(todo)
                }
                Err(e) => res.bad_request(&e.to_string()),
            }
        }
    });

    // Update todo
    let db_clone = db.clone();
    app.put("/todos/:id", move |req, res| {
        let db = db_clone.clone();
        async move {
            let id = req.param("id").unwrap().clone();
            
            match req.json::<Todo>() {
                Ok(mut todo) => {
                    todo.id = id.clone();
                    db.write().insert(id, todo.clone());
                    res.json(todo)
                }
                Err(e) => res.bad_request(&e.to_string()),
            }
        }
    });

    // Delete todo
    let db_clone = db.clone();
    app.delete("/todos/:id", move |req, res| {
        let db = db_clone.clone();
        async move {
            let id = req.param("id").unwrap();
            match db.write().remove(id) {
                Some(_) => res.no_content(),
                None => res.not_found(),
            }
        }
    });

    app.listen(3000).await
}
```

---

## Adding Middleware

```rust
use rustyx::prelude::*;
use rustyx::middleware::*;

#[tokio::main]
async fn main() -> Result<()> {
    let app = RustyX::new();

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
use rustyx::prelude::*;

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
    let app = RustyX::new();

    // Mount router at /api/users
    app.use_router("/api/users", user_routes());

    app.listen(3000).await
}
```

---

## Next Steps

- [Middleware Guide](./MIDDLEWARE.md) - Learn about built-in and custom middleware
- [Database Guide](./DATABASE.md) - Connect to databases
- [API Reference](./API.md) - Complete API documentation
- [Deployment Guide](./DEPLOYMENT.md) - Deploy your application
