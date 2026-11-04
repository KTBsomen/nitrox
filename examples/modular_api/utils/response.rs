//! Standardized JSON API Response Format

use nitrox::Response;
use serde::Serialize;
use serde_json::json;

#[allow(dead_code)]
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub message: Option<String>,
    pub data: T,
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct PaginationMeta {
    pub total: usize,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

pub fn ok<T: Serialize>(res: Response, data: T) -> Response {
    res.json(json!({
        "success": true,
        "data": data
    }))
}

pub fn created<T: Serialize>(res: Response, message: &str, data: T) -> Response {
    res.status(201).json(json!({
        "success": true,
        "message": message,
        "data": data
    }))
}

pub fn paginated<T: Serialize>(
    res: Response,
    items: Vec<T>,
    total: usize,
    page: u32,
    limit: u32,
) -> Response {
    let total_pages = if limit == 0 { 1 } else { (total as f64 / limit as f64).ceil() as u32 };
    res.json(json!({
        "success": true,
        "data": items,
        "pagination": {
            "total": total,
            "page": page,
            "limit": limit,
            "total_pages": total_pages
        }
    }))
}
