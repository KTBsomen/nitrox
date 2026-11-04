//! Authentication Models & DTOs

use super::user::{User, UserRole};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterDto {
    pub name: String,
    pub email: String,
    pub password: String,
    pub age: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub token_type: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: String,
    pub email: String,
    pub role: UserRole,
    pub exp: i64,
}
