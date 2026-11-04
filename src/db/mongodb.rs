//! MongoDB Module

use crate::error::Result;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

/// MongoDB Repository trait
#[async_trait]
pub trait MongoRepository<T>: Send + Sync
where
    T: Send + Sync + Serialize + DeserializeOwned,
{
    /// Find a document by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<T>>;

    /// Find all documents
    async fn find_all(&self) -> Result<Vec<T>>;

    /// Find documents matching a filter
    async fn find(&self, filter: serde_json::Value) -> Result<Vec<T>>;

    /// Find one document matching a filter
    async fn find_one(&self, filter: serde_json::Value) -> Result<Option<T>>;

    /// Insert a document
    async fn insert_one(&self, doc: &T) -> Result<String>;

    /// Insert many documents
    async fn insert_many(&self, docs: &[T]) -> Result<Vec<String>>;

    /// Update a document
    async fn update_one(&self, id: &str, update: serde_json::Value) -> Result<bool>;

    /// Update many documents
    async fn update_many(
        &self,
        filter: serde_json::Value,
        update: serde_json::Value,
    ) -> Result<u64>;

    /// Delete a document
    async fn delete_one(&self, id: &str) -> Result<bool>;

    /// Delete many documents
    async fn delete_many(&self, filter: serde_json::Value) -> Result<u64>;

    /// Count documents
    async fn count(&self, filter: Option<serde_json::Value>) -> Result<u64>;
}

/// MongoDB aggregation pipeline builder
pub struct AggregationBuilder {
    stages: Vec<serde_json::Value>,
}

impl AggregationBuilder {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn match_stage(mut self, filter: serde_json::Value) -> Self {
        self.stages.push(serde_json::json!({ "$match": filter }));
        self
    }

    pub fn group(mut self, group: serde_json::Value) -> Self {
        self.stages.push(serde_json::json!({ "$group": group }));
        self
    }

    pub fn sort(mut self, sort: serde_json::Value) -> Self {
        self.stages.push(serde_json::json!({ "$sort": sort }));
        self
    }

    pub fn limit(mut self, n: i64) -> Self {
        self.stages.push(serde_json::json!({ "$limit": n }));
        self
    }

    pub fn skip(mut self, n: i64) -> Self {
        self.stages.push(serde_json::json!({ "$skip": n }));
        self
    }

    pub fn build(self) -> Vec<serde_json::Value> {
        self.stages
    }
}

impl Default for AggregationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
