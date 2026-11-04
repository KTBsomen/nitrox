//! Database Module
//!
//! Provides ORM-like database abstraction for MongoDB, MySQL, SQLite, and PostgreSQL.

pub mod connection;
pub mod pool;
pub mod query;

#[cfg(feature = "mongodb")]
pub mod mongodb;

#[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
pub mod sql;

use crate::error::Result;
use async_trait::async_trait;

/// Prelude for database imports
pub mod prelude {
    pub use super::connection::*;
    pub use super::pool::*;
    pub use super::query::*;
    pub use super::{Database, DatabaseConfig, DbDriver};

    #[cfg(feature = "mongodb")]
    pub use super::mongodb::*;

    #[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
    pub use super::sql::*;
}

/// Supported database drivers
#[derive(Debug, Clone, PartialEq)]
pub enum DbDriver {
    MySQL,
    PostgreSQL,
    SQLite,
    MongoDB,
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub driver: DbDriver,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub fn new(driver: DbDriver, database: &str) -> Self {
        let (host, port) = match driver {
            DbDriver::MySQL => ("localhost".to_string(), 3306),
            DbDriver::PostgreSQL => ("localhost".to_string(), 5432),
            DbDriver::MongoDB => ("localhost".to_string(), 27017),
            DbDriver::SQLite => (String::new(), 0),
        };

        Self {
            driver,
            host,
            port,
            username: String::new(),
            password: String::new(),
            database: database.to_string(),
            max_connections: 10,
        }
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    pub fn username(mut self, username: &str) -> Self {
        self.username = username.to_string();
        self
    }
    pub fn password(mut self, password: &str) -> Self {
        self.password = password.to_string();
        self
    }
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    pub fn connection_string(&self) -> String {
        match self.driver {
            DbDriver::MySQL => format!(
                "mysql://{}:{}@{}:{}/{}",
                self.username, self.password, self.host, self.port, self.database
            ),
            DbDriver::PostgreSQL => format!(
                "postgres://{}:{}@{}:{}/{}",
                self.username, self.password, self.host, self.port, self.database
            ),
            DbDriver::SQLite => format!("sqlite:{}", self.database),
            DbDriver::MongoDB => format!(
                "mongodb://{}:{}@{}:{}/{}",
                self.username, self.password, self.host, self.port, self.database
            ),
        }
    }
}

/// Main database trait
#[async_trait]
pub trait Database: Send + Sync {
    async fn connect(config: &DatabaseConfig) -> Result<Self>
    where
        Self: Sized;
    async fn disconnect(&self) -> Result<()>;
    async fn is_connected(&self) -> bool;
}
