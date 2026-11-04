//! RustyX - Main Entry
//!
//! This is the main entry point for development/testing.

use rustyx::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting RustyX development server...");

    let app = RustyX::new();

    // Welcome route
    app.get("/", |_req, res| async move {
        res.json(json!({
            "name": "RustyX",
            "version": rustyx::VERSION,
            "description": "A fast, minimalist web framework for Rust"
        }))
    });

    // Start server
    app.listen(3000).await
}
