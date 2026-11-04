//! Bearer Authentication Guard Middleware

use crate::services::AppState;
use rustyx::middleware::Next;
use rustyx::{Request, Response};
use serde_json::json;
use std::sync::Arc;

#[allow(dead_code)]
pub fn require_auth(
    state: Arc<AppState>,
) -> impl Fn(Request, Response, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Send + Sync + 'static {
    move |req: Request, res: Response, next: Next| {
        let state = Arc::clone(&state);
        Box::pin(async move {
            let auth_header = req.header("authorization");
            match auth_header {
                Some(header_val) if header_val.starts_with("Bearer ") => {
                    match state.auth_service.verify_token(header_val) {
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
        })
    }
}
