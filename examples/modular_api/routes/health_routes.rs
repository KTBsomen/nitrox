//! Health & Monitoring Routes Module

use crate::controllers::HealthController;
use crate::services::AppState;
use nitrox::Router;

pub fn router() -> Router<AppState> {
    let mut r = Router::with_state();

    r.get("/health", |req, res| async move {
        HealthController::health_check(req, res).await
    });

    r.get("/metrics", |req, res| async move {
        HealthController::metrics(req, res).await
    });

    r
}
