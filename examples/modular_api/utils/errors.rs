//! Standardized Application Error Types

use rustyx::Response;
use serde_json::json;

#[allow(dead_code)]
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    Internal(String),
}

impl AppError {
    pub fn to_response(&self, res: Response) -> Response {
        match self {
            AppError::NotFound(msg) => res.status(404).json(json!({
                "success": false,
                "error": {
                    "code": "NOT_FOUND",
                    "message": msg
                }
            })),
            AppError::BadRequest(msg) => res.status(400).json(json!({
                "success": false,
                "error": {
                    "code": "BAD_REQUEST",
                    "message": msg
                }
            })),
            AppError::Unauthorized(msg) => res.status(401).json(json!({
                "success": false,
                "error": {
                    "code": "UNAUTHORIZED",
                    "message": msg
                }
            })),
            AppError::Forbidden(msg) => res.status(403).json(json!({
                "success": false,
                "error": {
                    "code": "FORBIDDEN",
                    "message": msg
                }
            })),
            AppError::Conflict(msg) => res.status(409).json(json!({
                "success": false,
                "error": {
                    "code": "CONFLICT",
                    "message": msg
                }
            })),
            AppError::Internal(msg) => res.status(500).json(json!({
                "success": false,
                "error": {
                    "code": "INTERNAL_SERVER_ERROR",
                    "message": msg
                }
            })),
        }
    }
}
