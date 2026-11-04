//! Models Module
//!
//! Provides base model traits and utilities for database entities.

use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

/// Base Model trait that all models should implement
#[async_trait]
pub trait Model: Send + Sync + Serialize + DeserializeOwned + Clone {
    /// Get the collection/table name
    fn collection_name() -> &'static str;

    /// Get the primary key field name
    fn primary_key() -> &'static str {
        "id"
    }

    /// Get the model's ID
    fn get_id(&self) -> Option<String>;

    /// Set the model's ID
    fn set_id(&mut self, id: String);

    /// Validate the model
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    /// Called before saving
    fn before_save(&mut self) {}

    /// Called after saving
    fn after_save(&mut self) {}

    /// Called before deleting
    fn before_delete(&self) {}

    /// Called after deleting
    fn after_delete(&self) {}
}

/// Trait for models with timestamps
pub trait Timestamps {
    fn created_at(&self) -> Option<DateTime<Utc>>;
    fn updated_at(&self) -> Option<DateTime<Utc>>;
    fn set_created_at(&mut self, time: DateTime<Utc>);
    fn set_updated_at(&mut self, time: DateTime<Utc>);
}

/// Trait for soft-deletable models
pub trait SoftDeletes {
    fn deleted_at(&self) -> Option<DateTime<Utc>>;
    fn set_deleted_at(&mut self, time: Option<DateTime<Utc>>);
    fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }
    fn soft_delete(&mut self) {
        self.set_deleted_at(Some(Utc::now()));
    }
    fn restore(&mut self) {
        self.set_deleted_at(None);
    }
}

/// Base entity with common fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEntity {
    #[serde(default = "generate_uuid")]
    pub id: String,
    #[serde(default = "now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "now")]
    pub updated_at: DateTime<Utc>,
}

fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}
fn now() -> DateTime<Utc> {
    Utc::now()
}

impl Default for BaseEntity {
    fn default() -> Self {
        Self {
            id: generate_uuid(),
            created_at: now(),
            updated_at: now(),
        }
    }
}

/// Schema field types
#[derive(Debug, Clone)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Json,
    Uuid,
    Text,
    Binary,
}

/// Schema field definition
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub unique: bool,
    pub default: Option<String>,
}

impl Field {
    pub fn new(name: &str, field_type: FieldType) -> Self {
        Self {
            name: name.to_string(),
            field_type,
            required: false,
            unique: false,
            default: None,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }
    pub fn default(mut self, value: &str) -> Self {
        self.default = Some(value.to_string());
        self
    }
}
