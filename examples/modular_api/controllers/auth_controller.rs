//! Authentication Controller

use crate::models::{LoginDto, RegisterDto};
use crate::services::AppState;
use crate::utils::{created, ok};
use nitrox::{Request, Response};
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthController {
    state: Arc<AppState>,
}

impl AuthController {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn login(&self, req: Request, res: Response) -> Response {
        match req.json::<LoginDto>() {
            Ok(dto) => match self.state.auth_service.login(dto).await {
                Ok(auth_res) => ok(res, auth_res),
                Err(err) => err.to_response(res),
            },
            Err(e) => res.bad_request(&format!("Invalid login credentials format: {}", e)),
        }
    }

    pub async fn register(&self, req: Request, res: Response) -> Response {
        match req.json::<RegisterDto>() {
            Ok(dto) => match self.state.auth_service.register(dto).await {
                Ok(auth_res) => created(res, "User registered successfully", auth_res),
                Err(err) => err.to_response(res),
            },
            Err(e) => res.bad_request(&format!("Invalid registration data: {}", e)),
        }
    }

    pub async fn me(&self, req: Request, res: Response) -> Response {
        // Read authorization header
        let auth_header = req.header("authorization").unwrap_or_default();
        match self.state.auth_service.verify_token(auth_header) {
            Ok(claims) => match self.state.user_service.get_by_id(&claims.sub).await {
                Ok(user) => ok(res, user),
                Err(err) => err.to_response(res),
            },
            Err(err) => err.to_response(res),
        }
    }
}
