//! Production Multi-File Modular REST API
//!
//! Demonstrates a full enterprise architecture built with Nitrox:
//! - Config (Environment-driven configuration)
//! - Models & DTOs (Serde serialization & validation)
//! - Services (Thread-safe business logic & state)
//! - Controllers (HTTP request handlers)
//! - Middleware (Logging, Request ID, Error Handling)
//! - Routes (Modular sub-routers mounted with prefixes)

mod config;
mod controllers;
mod middleware;
mod models;
mod routes;
mod services;
mod utils;

use config::AppConfig;
use nitrox::prelude::*;
use services::AppState;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,nitrox=info".to_string()),
        )
        .init();

    // 2. Load environment configuration
    let config = AppConfig::from_env();
    info!("🔧 Configuration loaded for environment: {:?}", config.environment);

    // 3. Initialize Shared Application State & Services
    let state = AppState::new(config.clone());

    // 4. Initialize Nitrox application
    let app = Nitrox::new();

    // 5. Global Middlewares
    app.use_middleware(middleware::request_id_middleware);

    // 6. Root Route - API Index & Discovery
    app.get("/", |_req, res| async move {
        res.json(json!({
            "service": "Nitrox Enterprise REST API",
            "version": nitrox::VERSION,
            "status": "operational",
            "endpoints": {
                "health": "/health",
                "metrics": "/metrics",
                "auth": {
                    "login": "POST /api/v1/auth/login",
                    "register": "POST /api/v1/auth/register",
                    "profile": "GET /api/v1/auth/me"
                },
                "users": {
                    "list": "GET /api/v1/users?page=1&limit=10",
                    "create": "POST /api/v1/users",
                    "get": "GET /api/v1/users/:id",
                    "update": "PUT /api/v1/users/:id",
                    "delete": "DELETE /api/v1/users/:id"
                },
                "products": {
                    "list": "GET /api/v1/products?q=keyboard&category=electronics",
                    "create": "POST /api/v1/products",
                    "get": "GET /api/v1/products/:id",
                    "update": "PUT /api/v1/products/:id",
                    "delete": "DELETE /api/v1/products/:id"
                }
            }
        }))
    });

    // 7. Mount Modular Routes
    routes::configure_routes(&app, state);

    // 8. Start HTTP Server
    info!("🚀 Starting {} server on port {}...", config.app_name, config.server_port);
    app.listen(config.server_port).await
}
