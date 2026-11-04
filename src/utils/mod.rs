//! Utilities Module
//!
//! Provides helper functions and utilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pagination helper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}

impl Pagination {
    pub fn new(page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;
        Self {
            page,
            per_page,
            total,
            total_pages,
        }
    }

    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.per_page
    }
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }
    pub fn has_prev(&self) -> bool {
        self.page > 1
    }
}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: Pagination,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, pagination: Pagination) -> Self {
        Self { data, pagination }
    }
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
            meta: None,
        }
    }

    pub fn with_meta(mut self, key: &str, value: serde_json::Value) -> Self {
        self.meta
            .get_or_insert_with(HashMap::new)
            .insert(key.to_string(), value);
        self
    }
}

/// Validation helpers
pub mod validation {
    pub fn is_email(value: &str) -> bool {
        value.contains('@') && value.contains('.')
    }

    pub fn min_length(value: &str, min: usize) -> bool {
        value.len() >= min
    }
    pub fn max_length(value: &str, max: usize) -> bool {
        value.len() <= max
    }
    pub fn is_numeric(value: &str) -> bool {
        value.chars().all(|c| c.is_numeric())
    }
    pub fn is_alphanumeric(value: &str) -> bool {
        value.chars().all(|c| c.is_alphanumeric())
    }
}

/// Hash password (placeholder - use bcrypt/argon2 in production)
pub fn hash_password(password: &str) -> String {
    format!("hashed_{}", password) // Replace with actual hashing
}

/// Verify password
pub fn verify_password(password: &str, hash: &str) -> bool {
    hash == format!("hashed_{}", password)
}

/// Generate random string
pub fn random_string(length: usize) -> String {
    use std::iter;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = std::collections::hash_map::RandomState::new();
    iter::repeat_with(|| CHARSET[rand_index(&mut rng, CHARSET.len())])
        .map(char::from)
        .take(length)
        .collect()
}

fn rand_index(state: &mut std::collections::hash_map::RandomState, max: usize) -> usize {
    use std::hash::{BuildHasher, Hasher};
    (state.build_hasher().finish() as usize) % max
}
