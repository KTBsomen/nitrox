//! Basic API Example
//!
//! This example demonstrates how to create a simple REST API using RustyX.

use rustyx::prelude::*;

// Define a User model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<String>,
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub age: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    // Create the application
    let app = RustyX::new();

    // Root route
    app.get("/", |_req, res| async move {
        res.json(json!({
            "message": "Welcome to RustyX!",
            "version": rustyx::VERSION
        }))
    });

    // Health check
    app.get("/health", |_req, res| async move {
        res.json(json!({ "status": "ok" }))
    });

    // API Routes - List users
    app.get("/api/users", |_req, res| async move {
        let users = vec![
            json!({ "id": "1", "name": "John Doe", "email": "john@example.com" }),
            json!({ "id": "2", "name": "Jane Doe", "email": "jane@example.com" }),
        ];
        res.json(json!({ "data": users }))
    });

    // Get single user
    app.get("/api/users/:id", |req, res| async move {
        let id = req.param("id").cloned().unwrap_or_default();
        res.json(json!({
            "id": id,
            "name": "John Doe",
            "email": "john@example.com"
        }))
    });

    // Create user
    app.post("/api/users", |req, res| async move {
        match req.json::<User>() {
            Ok(mut user) => {
                user.id = Some(uuid::Uuid::new_v4().to_string());
                res.status(201).json(json!({
                    "message": "User created",
                    "data": user
                }))
            }
            Err(e) => res.bad_request(&e.to_string()),
        }
    });

    // Update user
    app.put("/api/users/:id", |req, res| async move {
        let id = req.param("id").cloned().unwrap_or_default();
        match req.json::<User>() {
            Ok(mut user) => {
                user.id = Some(id);
                res.json(json!({
                    "message": "User updated",
                    "data": user
                }))
            }
            Err(e) => res.bad_request(&e.to_string()),
        }
    });

    // Delete user
    app.delete("/api/users/:id", |req, res| async move {
        let id = req.param("id").cloned().unwrap_or_default();
        res.json(json!({
            "message": "User deleted",
            "id": id
        }))
    });

    // Start the server
    info!("🚀 Starting RustyX server...");
    app.listen(3000).await
}
