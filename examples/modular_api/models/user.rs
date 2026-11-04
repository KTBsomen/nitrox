//! User Domain Models & DTOs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    Customer,
    Manager,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
    pub age: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserDto {
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    pub role: Option<UserRole>,
    pub age: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserDto {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<UserRole>,
    pub age: Option<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct UserSummary {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: UserRole,
}

impl From<&User> for UserSummary {
    fn from(user: &User) -> Self {
        Self {
            id: user.id.clone(),
            name: user.name.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
        }
    }
}
