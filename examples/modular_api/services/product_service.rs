//! Product Catalog Service

use crate::models::{CreateProductDto, Product, ProductCategory, ProductFilterQuery, UpdateProductDto};
use crate::utils::AppError;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct ProductService {
    products: Arc<RwLock<HashMap<String, Product>>>,
}

impl ProductService {
    pub fn new() -> Self {
        let mut initial = HashMap::new();

        let p1_id = "prod-1".to_string();
        initial.insert(
            p1_id.clone(),
            Product {
                id: p1_id,
                title: "Mechanical Keyboard".to_string(),
                description: "Wireless mechanical keyboard with RGB backlighting".to_string(),
                price: 119.99,
                stock: 45,
                category: ProductCategory::Electronics,
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );

        let p2_id = "prod-2".to_string();
        initial.insert(
            p2_id.clone(),
            Product {
                id: p2_id,
                title: "Ergonomic Office Chair".to_string(),
                description: "Breathable mesh ergonomic chair with lumbar support".to_string(),
                price: 249.50,
                stock: 12,
                category: ProductCategory::Home,
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );

        let p3_id = "prod-3".to_string();
        initial.insert(
            p3_id.clone(),
            Product {
                id: p3_id,
                title: "Programming Rust (2nd Edition)".to_string(),
                description: "Fast, Safe Systems Development by Jim Blandy".to_string(),
                price: 49.99,
                stock: 80,
                category: ProductCategory::Books,
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );

        Self {
            products: Arc::new(RwLock::new(initial)),
        }
    }

    pub async fn query(&self, filter: ProductFilterQuery) -> (Vec<Product>, usize) {
        let store = self.products.read().await;
        let mut list: Vec<Product> = store
            .values()
            .filter(|p| {
                if !p.is_active {
                    return false;
                }
                if let Some(ref search) = filter.search {
                    let needle = search.to_lowercase();
                    if !p.title.to_lowercase().contains(&needle)
                        && !p.description.to_lowercase().contains(&needle)
                    {
                        return false;
                    }
                }
                if let Some(ref cat_str) = filter.category {
                    let cat_match = match cat_str.to_lowercase().as_str() {
                        "electronics" => p.category == ProductCategory::Electronics,
                        "clothing" => p.category == ProductCategory::Clothing,
                        "books" => p.category == ProductCategory::Books,
                        "home" => p.category == ProductCategory::Home,
                        _ => true,
                    };
                    if !cat_match {
                        return false;
                    }
                }
                if let Some(min) = filter.min_price {
                    if p.price < min {
                        return false;
                    }
                }
                if let Some(max) = filter.max_price {
                    if p.price > max {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let total = list.len();
        let skip = ((filter.page.saturating_sub(1)) * filter.limit) as usize;
        let paged = list.into_iter().skip(skip).take(filter.limit as usize).collect();

        (paged, total)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Product, AppError> {
        let store = self.products.read().await;
        store
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Product with id '{}' not found", id)))
    }

    pub async fn create(&self, dto: CreateProductDto) -> Result<Product, AppError> {
        if dto.title.trim().is_empty() {
            return Err(AppError::BadRequest("Product title is required".to_string()));
        }
        if dto.price <= 0.0 {
            return Err(AppError::BadRequest("Product price must be greater than 0".to_string()));
        }

        let mut store = self.products.write().await;
        let id = format!("prd_{}", Uuid::new_v4().simple());
        let now = Utc::now();
        let product = Product {
            id: id.clone(),
            title: dto.title,
            description: dto.description.unwrap_or_default(),
            price: dto.price,
            stock: dto.stock.unwrap_or(0),
            category: dto.category.unwrap_or(ProductCategory::Other),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        store.insert(id, product.clone());
        Ok(product)
    }

    pub async fn update(&self, id: &str, dto: UpdateProductDto) -> Result<Product, AppError> {
        let mut store = self.products.write().await;
        let product = store
            .get_mut(id)
            .ok_or_else(|| AppError::NotFound(format!("Product with id '{}' not found", id)))?;

        if let Some(title) = dto.title {
            if title.trim().is_empty() {
                return Err(AppError::BadRequest("Product title cannot be empty".to_string()));
            }
            product.title = title;
        }

        if let Some(desc) = dto.description {
            product.description = desc;
        }

        if let Some(price) = dto.price {
            if price <= 0.0 {
                return Err(AppError::BadRequest("Price must be greater than 0".to_string()));
            }
            product.price = price;
        }

        if let Some(stock) = dto.stock {
            product.stock = stock;
        }

        if let Some(category) = dto.category {
            product.category = category;
        }

        if let Some(is_active) = dto.is_active {
            product.is_active = is_active;
        }

        product.updated_at = Utc::now();
        Ok(product.clone())
    }

    pub async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let mut store = self.products.write().await;
        if store.remove(id).is_some() {
            Ok(true)
        } else {
            Err(AppError::NotFound(format!("Product with id '{}' not found", id)))
        }
    }
}
