//! Health and System Metrics Controller

use crate::services::AppState;
use nitrox::{Request, Response};
use serde_json::json;

pub struct HealthController;

impl HealthController {
    pub async fn health_check(req: Request<AppState>, res: Response) -> Response {
        res.json(json!({
            "status": "healthy",
            "uptime_seconds": req.state.start_time.elapsed().as_secs(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    pub async fn metrics(req: Request<AppState>, res: Response) -> Response {
        res.json(json!({
            "app_name": req.state.config.app_name,
            "environment": format!("{:?}", req.state.config.environment),
            "uptime_seconds": req.state.start_time.elapsed().as_secs(),
            "framework": "Nitrox",
            "version": nitrox::VERSION
        }))
    }
}
