//! Authentication Routes Module

use crate::controllers::AuthController;
use crate::services::AppState;
use nitrox::Router;
use std::sync::Arc;

pub fn router(state: Arc<AppState>) -> Router {
    let mut r = Router::new();
    let controller = AuthController::new(state);

    // POST /login
    let c = controller.clone();
    r.post("/login", move |req, res| {
        let c = c.clone();
        async move { c.login(req, res).await }
    });

    // POST /register
    let c = controller.clone();
    r.post("/register", move |req, res| {
        let c = c.clone();
        async move { c.register(req, res).await }
    });

    // GET /me (Current User Profile)
    let c = controller.clone();
    r.get("/me", move |req, res| {
        let c = c.clone();
        async move { c.me(req, res).await }
    });

    r
}
