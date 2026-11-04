//! Error Module
//!
//! Provides error types and Result alias for the framework.

use thiserror::Error;

/// Custom error type for RustyX
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] hyper::Error),

    #[error("HTTP error: {0}")]
    HyperHttp(#[from] hyper::http::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn status_code(&self) -> u16 {
        match self {
            Error::NotFound(_) => 404,
            Error::Unauthorized(_) => 401,
            Error::Forbidden(_) => 403,
            Error::BadRequest(_) | Error::Validation(_) | Error::ParseError(_) => 400,
            _ => 500,
        }
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Error::NotFound(msg.into())
    }
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Error::BadRequest(msg.into())
    }
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Error::Unauthorized(msg.into())
    }
    pub fn database(msg: impl Into<String>) -> Self {
        Error::Database(msg.into())
    }
}

impl From<Error> for crate::response::Response {
    fn from(error: Error) -> Self {
        crate::response::Response::new()
            .status(error.status_code())
            .json(serde_json::json!({ "error": error.to_string() }))
    }
}

// Body collection errors are handled inline where they occur
