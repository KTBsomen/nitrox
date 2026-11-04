//! Health & Monitoring Routes Module

use crate::controllers::HealthController;
use crate::services::AppState;
use nitrox::Router;
use std::sync::Arc;

pub fn router(state: Arc<AppState>) -> Router {
    let mut r = Router::new();
    let controller = HealthController::new(state);

    // GET /health
    let c = controller.clone();
    r.get("/health", move |req, res| {
        let c = c.clone();
        async move { c.health_check(req, res).await }
    });

    // GET /metrics
    let c = controller.clone();
    r.get("/metrics", move |req, res| {
        let c = c.clone();
        async move { c.metrics(req, res).await }
    });

    r
}
