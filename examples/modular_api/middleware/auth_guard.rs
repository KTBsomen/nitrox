//! Bearer Authentication Guard Middleware

use crate::services::AppState;
use nitrox::middleware::Next;
use nitrox::{Request, Response};
use serde_json::json;

#[allow(dead_code)]
pub async fn require_auth(
    req: Request<AppState>,
    res: Response,
    next: Next<AppState>,
) -> Response {
    let auth_header = req.header("authorization");
    match auth_header {
        Some(header_val) if header_val.starts_with("Bearer ") => {
            match req.state.auth_service.verify_token(header_val) {
                Ok(_claims) => next(req, res).await,
                Err(err) => err.to_response(res),
            }
        }
        _ => res.status(401).json(json!({
            "success": false,
            "error": {
                "code": "UNAUTHORIZED",
                "message": "Missing or invalid Authorization header. Expected 'Bearer <token>'"
            }
        })),
    }
}
