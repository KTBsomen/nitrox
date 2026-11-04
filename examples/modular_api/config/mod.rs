//! Application Configuration Module
//!
//! Loads configuration from environment variables with sensible production defaults.

use std::env;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
    pub app_name: String,
    pub environment: Environment,
    pub jwt_secret: String,
    pub rate_limit_rpm: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            _ => Environment::Development,
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);
        let app_name = env::var("APP_NAME").unwrap_or_else(|_| "RustyX-Store-API".to_string());
        let environment = Environment::from_str(&env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()));
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "super-secret-production-key-change-me".to_string());
        let rate_limit_rpm = env::var("RATE_LIMIT_RPM")
            .ok()
            .and_then(|r| r.parse().ok())
            .unwrap_or(1000);

        Self {
            server_host,
            server_port,
            app_name,
            environment,
            jwt_secret,
            rate_limit_rpm,
        }
    }
}
