pub mod auth_service;
pub mod product_service;
pub mod user_service;

pub use auth_service::AuthService;
pub use product_service::ProductService;
pub use user_service::UserService;

use crate::config::AppConfig;
use std::time::Instant;

/// Global Shared Application State
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub user_service: UserService,
    pub product_service: ProductService,
    pub auth_service: AuthService,
    pub start_time: Instant,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let user_service = UserService::new();
        let product_service = ProductService::new();
        let auth_service = AuthService::new(user_service.clone(), config.jwt_secret.clone());

        Self {
            config,
            user_service,
            product_service,
            auth_service,
            start_time: Instant::now(),
        }
    }
}
