//! Authentication & Token Service

use crate::models::{AuthClaims, AuthResponse, CreateUserDto, LoginDto, RegisterDto, UserRole};
use crate::services::user_service::UserService;
use crate::utils::AppError;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthService {
    user_service: UserService,
    jwt_secret: Arc<String>,
}

impl AuthService {
    pub fn new(user_service: UserService, jwt_secret: String) -> Self {
        Self {
            user_service,
            jwt_secret: Arc::new(jwt_secret),
        }
    }

    pub async fn login(&self, dto: LoginDto) -> Result<AuthResponse, AppError> {
        let user = self
            .user_service
            .find_by_email(&dto.email)
            .await
            .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

        // Simple mock password verification (in production use argon2 / bcrypt)
        let expected_hash = format!("{}_hash", dto.password);
        if user.password_hash != expected_hash && user.password_hash != dto.password {
            return Err(AppError::Unauthorized("Invalid email or password".to_string()));
        }

        let token = self.generate_token(&user.id, &user.email, &user.role);

        Ok(AuthResponse {
            token,
            token_type: "Bearer".to_string(),
            user,
        })
    }

    pub async fn register(&self, dto: RegisterDto) -> Result<AuthResponse, AppError> {
        let password_hash = format!("{}_hash", dto.password);
        let user = self
            .user_service
            .create(CreateUserDto {
                name: dto.name,
                email: dto.email,
                password: Some(password_hash),
                role: Some(UserRole::Customer),
                age: dto.age,
            })
            .await?;

        let token = self.generate_token(&user.id, &user.email, &user.role);

        Ok(AuthResponse {
            token,
            token_type: "Bearer".to_string(),
            user,
        })
    }

    pub fn generate_token(&self, user_id: &str, email: &str, role: &UserRole) -> String {
        // Standard signed token format: user_id:email:role:expiry:signature
        let exp = Utc::now().timestamp() + 86400; // 24 hour expiry
        let payload = format!("{}:{}:{:?}:{}", user_id, email, role, exp);
        let sig = Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("{}:{}", payload, self.jwt_secret).as_bytes());
        format!("rxt_{}_{}", payload, sig.simple())
    }

    pub fn verify_token(&self, token: &str) -> Result<AuthClaims, AppError> {
        let stripped = token.strip_prefix("Bearer ").unwrap_or(token);
        if !stripped.starts_with("rxt_") {
            return Err(AppError::Unauthorized("Invalid token format".to_string()));
        }

        let parts: Vec<&str> = stripped.trim_start_matches("rxt_").split('_').collect();
        if parts.len() < 2 {
            return Err(AppError::Unauthorized("Malformed token".to_string()));
        }

        let payload = parts[0];
        let subparts: Vec<&str> = payload.split(':').collect();
        if subparts.len() < 4 {
            return Err(AppError::Unauthorized("Invalid token payload".to_string()));
        }

        let exp: i64 = subparts[3].parse().unwrap_or(0);
        if Utc::now().timestamp() > exp {
            return Err(AppError::Unauthorized("Token has expired".to_string()));
        }

        let role = match subparts[2] {
            "Admin" => UserRole::Admin,
            "Manager" => UserRole::Manager,
            _ => UserRole::Customer,
        };

        Ok(AuthClaims {
            sub: subparts[0].to_string(),
            email: subparts[1].to_string(),
            role,
            exp,
        })
    }
}
