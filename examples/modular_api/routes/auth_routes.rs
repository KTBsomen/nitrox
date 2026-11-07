//! Authentication Routes Module

use crate::controllers::AuthController;
use crate::services::AppState;
use nitrox::Router;

pub fn router() -> Router<AppState> {
    let mut r = Router::with_state();

    // POST /login
    r.post("/login", |req, res| async move {
        AuthController::login(req, res).await
    });

    // POST /register
    r.post("/register", |req, res| async move {
        AuthController::register(req, res).await
    });

    // GET /me (Current User Profile - Protected by route-level require_auth)
    r.get_with("/me", crate::middleware::require_auth, |req, res| async move {
        AuthController::me(req, res).await
    });

    r
}
