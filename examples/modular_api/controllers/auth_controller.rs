//! Authentication Controller

use crate::models::{LoginDto, RegisterDto};
use crate::services::AppState;
use crate::utils::{created, ok};
use nitrox::{Request, Response};

pub struct AuthController;

impl AuthController {
    pub async fn login(req: Request<AppState>, res: Response) -> Response {
        match req.json::<LoginDto>() {
            Ok(dto) => match req.state.auth_service.login(dto).await {
                Ok(auth_res) => ok(res, auth_res),
                Err(err) => err.to_response(res),
            },
            Err(e) => res.bad_request(&format!("Invalid login credentials format: {}", e)),
        }
    }

    pub async fn register(req: Request<AppState>, res: Response) -> Response {
        match req.json::<RegisterDto>() {
            Ok(dto) => match req.state.auth_service.register(dto).await {
                Ok(auth_res) => created(res, "User registered successfully", auth_res),
                Err(err) => err.to_response(res),
            },
            Err(e) => res.bad_request(&format!("Invalid registration data: {}", e)),
        }
    }

    pub async fn me(req: Request<AppState>, res: Response) -> Response {
        // Read authorization header
        let auth_header = req.header("authorization").unwrap_or_default();
        match req.state.auth_service.verify_token(auth_header) {
            Ok(claims) => match req.state.user_service.get_by_id(&claims.sub).await {
                Ok(user) => ok(res, user),
                Err(err) => err.to_response(res),
            },
            Err(err) => err.to_response(res),
        }
    }
}
