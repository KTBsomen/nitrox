//! Product Domain Models & DTOs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProductCategory {
    Electronics,
    Clothing,
    Books,
    Home,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub title: String,
    pub description: String,
    pub price: f64,
    pub stock: u32,
    pub category: ProductCategory,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductDto {
    pub title: String,
    pub description: Option<String>,
    pub price: f64,
    pub stock: Option<u32>,
    pub category: Option<ProductCategory>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductDto {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub stock: Option<u32>,
    pub category: Option<ProductCategory>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct ProductFilterQuery {
    pub search: Option<String>,
    pub category: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub page: u32,
    pub limit: u32,
}
