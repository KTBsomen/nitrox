//! User Routes Module

use crate::controllers::UserController;
use crate::services::AppState;
use nitrox::Router;

pub fn router() -> Router<AppState> {
    let mut r = Router::with_state();

    // List users (supports both / and without trailing slash)
    r.get("", |req, res| async move {
        UserController::list_users(req, res).await
    });
    r.get("/", |req, res| async move {
        UserController::list_users(req, res).await
    });

    // Create user
    r.post("", |req, res| async move {
        UserController::create_user(req, res).await
    });
    r.post("/", |req, res| async move {
        UserController::create_user(req, res).await
    });

    // Get user by ID
    r.get("/:id", |req, res| async move {
        UserController::get_user(req, res).await
    });

    // Update user by ID
    r.put("/:id", |req, res| async move {
        UserController::update_user(req, res).await
    });

    // Delete user by ID
    r.delete("/:id", |req, res| async move {
        UserController::delete_user(req, res).await
    });

    r
}
