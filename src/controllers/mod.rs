//! Controllers Module
//!
//! Provides base controller traits and utilities for handling requests.

#![allow(unused_variables)]

use crate::request::Request;
use crate::response::Response;
use async_trait::async_trait;

/// Base Controller trait
#[async_trait]
pub trait Controller: Send + Sync {
    /// Handle index/list request
    async fn index(&self, req: Request, res: Response) -> Response {
        res.status(501)
            .json(serde_json::json!({ "error": "Not implemented" }))
    }

    /// Handle show/get one request
    async fn show(&self, req: Request, res: Response) -> Response {
        res.status(501)
            .json(serde_json::json!({ "error": "Not implemented" }))
    }

    /// Handle create request
    async fn create(&self, req: Request, res: Response) -> Response {
        res.status(501)
            .json(serde_json::json!({ "error": "Not implemented" }))
    }

    /// Handle update request
    async fn update(&self, req: Request, res: Response) -> Response {
        res.status(501)
            .json(serde_json::json!({ "error": "Not implemented" }))
    }

    /// Handle delete request
    async fn destroy(&self, req: Request, res: Response) -> Response {
        res.status(501)
            .json(serde_json::json!({ "error": "Not implemented" }))
    }
}

/// CRUD Controller helper macro
#[macro_export]
macro_rules! crud_controller {
    ($name:ident, $model:ty, $collection:expr) => {
        pub struct $name;

        #[async_trait::async_trait]
        impl Controller for $name {
            async fn index(&self, _req: Request, res: Response) -> Response {
                res.json(serde_json::json!({ "data": [], "collection": $collection }))
            }

            async fn show(&self, req: Request, res: Response) -> Response {
                let id = req.param("id").cloned().unwrap_or_default();
                res.json(serde_json::json!({ "id": id, "collection": $collection }))
            }

            async fn create(&self, req: Request, res: Response) -> Response {
                match req.json::<$model>() {
                    Ok(data) => res.status(201).json(data),
                    Err(e) => res.bad_request(&e.to_string()),
                }
            }

            async fn update(&self, req: Request, res: Response) -> Response {
                let id = req.param("id").cloned().unwrap_or_default();
                match req.json::<$model>() {
                    Ok(data) => res.json(serde_json::json!({ "id": id, "data": data })),
                    Err(e) => res.bad_request(&e.to_string()),
                }
            }

            async fn destroy(&self, req: Request, res: Response) -> Response {
                let id = req.param("id").cloned().unwrap_or_default();
                res.json(serde_json::json!({ "deleted": true, "id": id }))
            }
        }
    };
}

/// Resource controller that auto-registers CRUD routes
pub struct ResourceController<C: Controller> {
    pub controller: C,
    pub path: String,
}

impl<C: Controller + 'static> ResourceController<C> {
    pub fn new(path: &str, controller: C) -> Self {
        Self {
            controller,
            path: path.to_string(),
        }
    }
}
