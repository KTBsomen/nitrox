//! Health and System Metrics Controller

use crate::services::AppState;
use nitrox::{Request, Response};
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct HealthController {
    #[allow(dead_code)]
    state: Arc<AppState>,
    start_time: Instant,
}

impl HealthController {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            start_time: Instant::now(),
        }
    }

    pub async fn health_check(&self, _req: Request, res: Response) -> Response {
        res.json(json!({
            "status": "healthy",
            "uptime_seconds": self.start_time.elapsed().as_secs(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    pub async fn metrics(&self, _req: Request, res: Response) -> Response {
        res.json(json!({
            "app_name": self.state.config.app_name,
            "environment": format!("{:?}", self.state.config.environment),
            "uptime_seconds": self.start_time.elapsed().as_secs(),
            "framework": "Nitrox",
            "version": nitrox::VERSION
        }))
    }
}
