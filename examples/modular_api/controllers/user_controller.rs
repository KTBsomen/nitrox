//! User Controller

use crate::models::{CreateUserDto, UpdateUserDto};
use crate::services::AppState;
use crate::utils::{created, ok, paginated};
use nitrox::{Request, Response};

pub struct UserController;

impl UserController {
    pub async fn list_users(req: Request<AppState>, res: Response) -> Response {
        let page: u32 = req.query_param("page").and_then(|v| v.parse().ok()).unwrap_or(1);
        let limit: u32 = req.query_param("limit").and_then(|v| v.parse().ok()).unwrap_or(10);

        let (users, total) = req.state.user_service.get_all(page, limit).await;
        paginated(res, users, total, page, limit)
    }

    pub async fn get_user(req: Request<AppState>, res: Response) -> Response {
        let id = req.param("id").cloned().unwrap_or_default();
        match req.state.user_service.get_by_id(&id).await {
            Ok(user) => ok(res, user),
            Err(err) => err.to_response(res),
        }
    }

    pub async fn create_user(req: Request<AppState>, res: Response) -> Response {
        match req.json::<CreateUserDto>() {
            Ok(dto) => match req.state.user_service.create(dto).await {
                Ok(user) => created(res, "User created successfully", user),
                Err(err) => err.to_response(res),
            },
            Err(e) => res.bad_request(&format!("Invalid JSON body: {}", e)),
        }
    }

    pub async fn update_user(req: Request<AppState>, res: Response) -> Response {
        let id = req.param("id").cloned().unwrap_or_default();
        match req.json::<UpdateUserDto>() {
            Ok(dto) => match req.state.user_service.update(&id, dto).await {
                Ok(user) => ok(res, user),
                Err(err) => err.to_response(res),
            },
            Err(e) => res.bad_request(&format!("Invalid JSON body: {}", e)),
        }
    }

    pub async fn delete_user(req: Request<AppState>, res: Response) -> Response {
        let id = req.param("id").cloned().unwrap_or_default();
        match req.state.user_service.delete(&id).await {
            Ok(_) => ok(res, serde_json::json!({ "message": "User deleted successfully", "id": id })),
            Err(err) => err.to_response(res),
        }
    }
}
