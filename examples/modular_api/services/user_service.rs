//! User Management Service (Business Logic & State)

use crate::models::{CreateUserDto, UpdateUserDto, User, UserRole};
use crate::utils::AppError;
use chrono::Utc;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserService {
    users: Arc<RwLock<HashMap<String, User>>>,
    emails: Arc<RwLock<HashMap<String, String>>>, // normalized_email -> user_id
}

impl UserService {
    pub fn new() -> Self {
        let mut initial_users = HashMap::new();
        let mut initial_emails = HashMap::new();

        // Seed initial mock users for testing
        let admin_id = "user-admin-1".to_string();
        let admin_email = "admin@example.com".to_string();
        initial_users.insert(
            admin_id.clone(),
            User {
                id: admin_id.clone(),
                name: "Admin User".to_string(),
                email: admin_email.clone(),
                password_hash: "admin123_hash".to_string(),
                role: UserRole::Admin,
                age: Some(35),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );
        initial_emails.insert(admin_email.to_lowercase(), admin_id);

        let customer_id = "user-customer-2".to_string();
        let customer_email = "john@example.com".to_string();
        initial_users.insert(
            customer_id.clone(),
            User {
                id: customer_id.clone(),
                name: "John Doe".to_string(),
                email: customer_email.clone(),
                password_hash: "pass123_hash".to_string(),
                role: UserRole::Customer,
                age: Some(28),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );
        initial_emails.insert(customer_email.to_lowercase(), customer_id);

        Self {
            users: Arc::new(RwLock::new(initial_users)),
            emails: Arc::new(RwLock::new(initial_emails)),
        }
    }

    pub async fn get_all(&self, page: u32, limit: u32) -> (Vec<User>, usize) {
        let mut list: Vec<User> = {
            let store = self.users.read();
            store.values().cloned().collect()
        };
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let total = list.len();
        let skip = ((page.saturating_sub(1)) * limit) as usize;
        let paged = list.into_iter().skip(skip).take(limit as usize).collect();

        (paged, total)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<User, AppError> {
        self.users
            .read()
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("User with id '{}' not found", id)))
    }

    pub async fn find_by_email(&self, email: &str) -> Option<User> {
        let normalized = email.to_lowercase();
        let user_id = self.emails.read().get(&normalized)?.clone();
        self.users.read().get(&user_id).cloned()
    }

    pub async fn create(&self, dto: CreateUserDto) -> Result<User, AppError> {
        if dto.name.trim().is_empty() {
            return Err(AppError::BadRequest("User name cannot be empty".to_string()));
        }
        if !dto.email.contains('@') {
            return Err(AppError::BadRequest("Invalid email address format".to_string()));
        }

        let normalized_email = dto.email.to_lowercase();

        // 1. O(1) Fast duplicate check using shared read-lock (eliminates write-lock contention)
        if self.emails.read().contains_key(&normalized_email) {
            return Err(AppError::Conflict(format!("Email '{}' is already in use", dto.email)));
        }

        // 2. Generate ID, timestamp, and User struct OUTSIDE all locks
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

        // 3. Atomically acquire write locks for the microsecond map insert only
        {
            let mut emails = self.emails.write();
            if emails.contains_key(&normalized_email) {
                return Err(AppError::Conflict(format!("Email '{}' is already in use", user.email)));
            }
            emails.insert(normalized_email, id.clone());
            self.users.write().insert(id, user.clone());
        }

        Ok(user)
    }

    pub async fn update(&self, id: &str, dto: UpdateUserDto) -> Result<User, AppError> {
        let mut users = self.users.write();
        let user = users
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
            let old_normalized = user.email.to_lowercase();
            let new_normalized = email.to_lowercase();
            if old_normalized != new_normalized {
                let mut emails = self.emails.write();
                if emails.contains_key(&new_normalized) {
                    return Err(AppError::Conflict(format!("Email '{}' is already in use", email)));
                }
                emails.remove(&old_normalized);
                emails.insert(new_normalized, id.to_string());
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
        let removed = self.users.write().remove(id);
        if let Some(user) = removed {
            self.emails.write().remove(&user.email.to_lowercase());
            Ok(true)
        } else {
            Err(AppError::NotFound(format!("User with id '{}' not found", id)))
        }
    }
}
