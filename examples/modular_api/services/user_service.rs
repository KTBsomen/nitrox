//! User Management Service (Business Logic & State)

use crate::models::{CreateUserDto, UpdateUserDto, User, UserRole};
use crate::utils::AppError;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserService {
    users: Arc<RwLock<HashMap<String, User>>>,
}

impl UserService {
    pub fn new() -> Self {
        let mut initial_users = HashMap::new();

        // Seed initial mock users for testing
        let admin_id = "user-admin-1".to_string();
        initial_users.insert(
            admin_id.clone(),
            User {
                id: admin_id,
                name: "Admin User".to_string(),
                email: "admin@example.com".to_string(),
                password_hash: "admin123_hash".to_string(),
                role: UserRole::Admin,
                age: Some(35),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );

        let customer_id = "user-customer-2".to_string();
        initial_users.insert(
            customer_id.clone(),
            User {
                id: customer_id,
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
                password_hash: "pass123_hash".to_string(),
                role: UserRole::Customer,
                age: Some(28),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );

        Self {
            users: Arc::new(RwLock::new(initial_users)),
        }
    }

    pub async fn get_all(&self, page: u32, limit: u32) -> (Vec<User>, usize) {
        let store = self.users.read().await;
        let mut list: Vec<User> = store.values().cloned().collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let total = list.len();
        let skip = ((page.saturating_sub(1)) * limit) as usize;
        let paged = list.into_iter().skip(skip).take(limit as usize).collect();

        (paged, total)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<User, AppError> {
        let store = self.users.read().await;
        store
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("User with id '{}' not found", id)))
    }

    pub async fn find_by_email(&self, email: &str) -> Option<User> {
        let store = self.users.read().await;
        store
            .values()
            .find(|u| u.email.eq_ignore_ascii_case(email))
            .cloned()
    }

    pub async fn create(&self, dto: CreateUserDto) -> Result<User, AppError> {
        if dto.name.trim().is_empty() {
            return Err(AppError::BadRequest("User name cannot be empty".to_string()));
        }
        if !dto.email.contains('@') {
            return Err(AppError::BadRequest("Invalid email address format".to_string()));
        }

        let mut store = self.users.write().await;
        if store.values().any(|u| u.email.eq_ignore_ascii_case(&dto.email)) {
            return Err(AppError::Conflict(format!("Email '{}' is already in use", dto.email)));
        }

        let id = format!("usr_{}", Uuid::new_v4().simple());
        let now = Utc::now();
        let user = User {
            id: id.clone(),
            name: dto.name,
            email: dto.email,
            password_hash: dto.password.unwrap_or_else(|| "default_hash".to_string()),
            role: dto.role.unwrap_or(UserRole::Customer),
            age: dto.age,
            created_at: now,
            updated_at: now,
        };

        store.insert(id, user.clone());
        Ok(user)
    }

    pub async fn update(&self, id: &str, dto: UpdateUserDto) -> Result<User, AppError> {
        let mut store = self.users.write().await;
        let user = store
            .get_mut(id)
            .ok_or_else(|| AppError::NotFound(format!("User with id '{}' not found", id)))?;

        if let Some(name) = dto.name {
            if name.trim().is_empty() {
                return Err(AppError::BadRequest("User name cannot be empty".to_string()));
            }
            user.name = name;
        }

        if let Some(email) = dto.email {
            if !email.contains('@') {
                return Err(AppError::BadRequest("Invalid email format".to_string()));
            }
            user.email = email;
        }

        if let Some(role) = dto.role {
            user.role = role;
        }

        if let Some(age) = dto.age {
            user.age = Some(age);
        }

        user.updated_at = Utc::now();
        Ok(user.clone())
    }

    pub async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let mut store = self.users.write().await;
        if store.remove(id).is_some() {
            Ok(true)
        } else {
            Err(AppError::NotFound(format!("User with id '{}' not found", id)))
        }
    }
}
