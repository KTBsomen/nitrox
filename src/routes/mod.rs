//! Routes Module
//!
//! Provides utilities for defining and organizing routes.

use crate::router::Router;

/// Trait for route definitions
pub trait RouteDefinition {
    /// Register routes on the router
    fn register(router: &mut Router);
}

/// Route group builder
pub struct RouteGroup {
    prefix: String,
    routes: Vec<RouteEntry>,
}

#[derive(Clone)]
pub struct RouteEntry {
    pub method: String,
    pub path: String,
    pub name: Option<String>,
}

impl RouteGroup {
    /// Create a new route group with a prefix
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            routes: Vec::new(),
        }
    }

    /// Add a GET route
    pub fn get(mut self, path: &str) -> Self {
        self.routes.push(RouteEntry {
            method: "GET".to_string(),
            path: format!("{}{}", self.prefix, path),
            name: None,
        });
        self
    }

    /// Add a POST route
    pub fn post(mut self, path: &str) -> Self {
        self.routes.push(RouteEntry {
            method: "POST".to_string(),
            path: format!("{}{}", self.prefix, path),
            name: None,
        });
        self
    }

    /// Add a PUT route
    pub fn put(mut self, path: &str) -> Self {
        self.routes.push(RouteEntry {
            method: "PUT".to_string(),
            path: format!("{}{}", self.prefix, path),
            name: None,
        });
        self
    }

    /// Add a DELETE route
    pub fn delete(mut self, path: &str) -> Self {
        self.routes.push(RouteEntry {
            method: "DELETE".to_string(),
            path: format!("{}{}", self.prefix, path),
            name: None,
        });
        self
    }

    /// Get all routes
    pub fn routes(&self) -> &[RouteEntry] {
        &self.routes
    }

    /// Get the prefix
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

/// API versioning helper
pub struct ApiVersion {
    pub version: String,
    pub prefix: String,
}

impl ApiVersion {
    pub fn new(version: u32) -> Self {
        Self {
            version: format!("v{}", version),
            prefix: format!("/api/v{}", version),
        }
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

/// Resource route helper - creates standard CRUD routes
pub fn resource_routes(name: &str) -> RouteGroup {
    RouteGroup::new(&format!("/{}", name))
        .get("") // index
        .get("/:id") // show
        .post("") // create
        .put("/:id") // update
        .delete("/:id") // delete
}
