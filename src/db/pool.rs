//! Database Pool Module
#![allow(dead_code)]
#![allow(clippy::mismatched_lifetime_syntaxes)]

use crate::db::DatabaseConfig;
use crate::error::Result;

/// Connection pool wrapper
pub struct Pool {
    config: DatabaseConfig,
    size: u32,
}

impl Pool {
    /// Create a new connection pool
    pub fn new(config: DatabaseConfig) -> Self {
        let size = config.max_connections;
        Self { config, size }
    }

    /// Get the pool size
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Get the configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Acquire a connection from the pool
    pub async fn acquire(&self) -> Result<PoolConnection<'_>> {
        Ok(PoolConnection { pool: self })
    }
}

/// A connection from the pool
pub struct PoolConnection<'a> {
    pool: &'a Pool,
}

impl<'a> PoolConnection<'a> {
    /// Release the connection back to the pool
    pub fn release(self) {
        // Connection is automatically released when dropped
    }
}
