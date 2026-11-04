//! Request ID Middleware

use rustyx::middleware::Next;
use rustyx::{Request, Response};
use uuid::Uuid;

pub async fn request_id_middleware(req: Request, res: Response, next: Next) -> Response {
    let request_id = req
        .header("x-request-id")
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("req_{}", Uuid::new_v4().simple()));

    let res = next(req, res).await;
    res.header("X-Request-Id", &request_id)
}
