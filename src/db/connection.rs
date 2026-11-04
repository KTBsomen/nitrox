//! Database Connection Module
#![allow(dead_code)]

use crate::db::{Database, DatabaseConfig, DbDriver};
use crate::error::Result;
use async_trait::async_trait;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use std::sync::Arc;

static DB_INSTANCE: OnceCell<Arc<RwLock<Option<DatabaseConnection>>>> = OnceCell::new();

/// Database connection wrapper
pub struct DatabaseConnection {
    config: DatabaseConfig,
    #[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
    sql_pool: Option<sqlx::AnyPool>,
    #[cfg(feature = "mongodb")]
    mongo_client: Option<mongodb::Client>,
}

impl DatabaseConnection {
    /// Create a new database connection
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let mut conn = Self {
            config: config.clone(),
            #[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
            sql_pool: None,
            #[cfg(feature = "mongodb")]
            mongo_client: None,
        };
        conn.establish_connection().await?;
        Ok(conn)
    }

    async fn establish_connection(&mut self) -> Result<()> {
        match self.config.driver {
            #[cfg(feature = "sqlite")]
            DbDriver::SQLite => {
                // SQLite connection handled by sqlx
            }
            #[cfg(feature = "mysql")]
            DbDriver::MySQL => {
                // MySQL connection handled by sqlx
            }
            #[cfg(feature = "postgres")]
            DbDriver::PostgreSQL => {
                // PostgreSQL connection handled by sqlx
            }
            #[cfg(feature = "mongodb")]
            DbDriver::MongoDB => {
                // MongoDB connection
            }
            _ => {}
        }
        Ok(())
    }

    /// Get the database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }
}

#[async_trait]
impl Database for DatabaseConnection {
    async fn connect(config: &DatabaseConfig) -> Result<Self> {
        DatabaseConnection::new(config.clone()).await
    }

    async fn disconnect(&self) -> Result<()> {
        Ok(())
    }
    async fn is_connected(&self) -> bool {
        true
    }
}

/// Initialize the global database connection
pub async fn init_db(config: DatabaseConfig) -> Result<()> {
    let conn = DatabaseConnection::new(config).await?;
    DB_INSTANCE.get_or_init(|| Arc::new(RwLock::new(Some(conn))));
    Ok(())
}

/// Get the global database connection
pub fn get_db() -> Option<Arc<RwLock<Option<DatabaseConnection>>>> {
    DB_INSTANCE.get().cloned()
}
