//! User Controller

use crate::models::{CreateUserDto, UpdateUserDto};
use crate::services::AppState;
use crate::utils::{ok, paginated};
use nitrox::{Request, Response};
use std::sync::Arc;

#[derive(Clone)]
pub struct UserController {
    state: Arc<AppState>,
}

impl UserController {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn list_users(&self, req: Request, res: Response) -> Response {
        let page: u32 = req.query_param("page").and_then(|v| v.parse().ok()).unwrap_or(1);
        let limit: u32 = req.query_param("limit").and_then(|v| v.parse().ok()).unwrap_or(10);

        let (users, total) = self.state.user_service.get_all(page, limit).await;
        paginated(res, users, total, page, limit)
    }

    pub async fn get_user(&self, req: Request, res: Response) -> Response {
        let id = req.param("id").cloned().unwrap_or_default();
        match self.state.user_service.get_by_id(&id).await {
            Ok(user) => ok(res, user),
            Err(err) => err.to_response(res),
        }
    }

    pub async fn create_user(&self, req: Request, res: Response) -> Response {
        match req.json::<CreateUserDto>() {
            Ok(dto) => match self.state.user_service.create(dto).await {
                Ok(user) => res.status(201).json(serde_json::json!({
                    "success": true,
                    "message": "User created successfully",
                    "data": user
                })),
                Err(err) => err.to_response(res),
            },
            Err(e) => res.bad_request(&format!("Invalid JSON body: {}", e)),
        }
    }

    pub async fn update_user(&self, req: Request, res: Response) -> Response {
        let id = req.param("id").cloned().unwrap_or_default();
        match req.json::<UpdateUserDto>() {
            Ok(dto) => match self.state.user_service.update(&id, dto).await {
                Ok(user) => ok(res, user),
                Err(err) => err.to_response(res),
            },
            Err(e) => res.bad_request(&format!("Invalid JSON body: {}", e)),
        }
    }

    pub async fn delete_user(&self, req: Request, res: Response) -> Response {
        let id = req.param("id").cloned().unwrap_or_default();
        match self.state.user_service.delete(&id).await {
            Ok(_) => ok(res, serde_json::json!({ "message": "User deleted successfully", "id": id })),
            Err(err) => err.to_response(res),
        }
    }
}
