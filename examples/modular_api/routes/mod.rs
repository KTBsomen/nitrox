//! Main Routing Module
//!
//! Mounts modular sub-routers onto the Nitrox application instance.

pub mod auth_routes;
pub mod health_routes;
pub mod product_routes;
pub mod user_routes;

use crate::services::AppState;
use nitrox::Nitrox;
use std::sync::Arc;

pub fn configure_routes(app: &Nitrox, state: Arc<AppState>) {
    // 1. Health & system monitoring endpoints
    app.use_router("", health_routes::router(state.clone()));

    // 2. Authentication endpoints: /api/v1/auth/*
    app.use_router("/api/v1/auth", auth_routes::router(state.clone()));

    // 3. User resource endpoints: /api/v1/users/*
    app.use_router("/api/v1/users", user_routes::router(state.clone()));

    // 4. Product catalog endpoints: /api/v1/products/*
    app.use_router("/api/v1/products", product_routes::router(state.clone()));

    // 5. Backwards-compatible routes for benchmarking: /api/users/*
    app.use_router("/api/users", user_routes::router(state.clone()));
}
