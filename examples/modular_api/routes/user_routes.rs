//! User Routes Module

use crate::controllers::UserController;
use crate::services::AppState;
use rustyx::Router;
use std::sync::Arc;

pub fn router(state: Arc<AppState>) -> Router {
    let mut r = Router::new();
    let controller = UserController::new(state);

    // List users (supports both / and without trailing slash)
    let c = controller.clone();
    r.get("", move |req, res| {
        let c = c.clone();
        async move { c.list_users(req, res).await }
    });
    let c = controller.clone();
    r.get("/", move |req, res| {
        let c = c.clone();
        async move { c.list_users(req, res).await }
    });

    // Create user
    let c = controller.clone();
    r.post("", move |req, res| {
        let c = c.clone();
        async move { c.create_user(req, res).await }
    });
    let c = controller.clone();
    r.post("/", move |req, res| {
        let c = c.clone();
        async move { c.create_user(req, res).await }
    });

    // Get user by ID
    let c = controller.clone();
    r.get("/:id", move |req, res| {
        let c = c.clone();
        async move { c.get_user(req, res).await }
    });

    // Update user by ID
    let c = controller.clone();
    r.put("/:id", move |req, res| {
        let c = c.clone();
        async move { c.update_user(req, res).await }
    });

    // Delete user by ID
    let c = controller.clone();
    r.delete("/:id", move |req, res| {
        let c = c.clone();
        async move { c.delete_user(req, res).await }
    });

    r
}
