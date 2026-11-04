//! Rate Limiting Middleware
//!
//! Provides rate limiting functionality to protect APIs from abuse.

use crate::middleware::Next;
use crate::request::Request;
use crate::response::Response;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Maximum number of requests allowed in the window
    pub max_requests: u32,
    /// Time window duration
    pub window: Duration,
    /// Custom message for rate limit exceeded
    pub message: String,
    /// Skip rate limiting for certain paths
    pub skip_paths: Vec<String>,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
            message: "Too many requests. Please try again later.".to_string(),
            skip_paths: vec![],
        }
    }
}

impl RateLimiterConfig {
    /// Create a new rate limiter config
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            max_requests,
            window: Duration::from_secs(window_secs),
            ..Default::default()
        }
    }

    /// Set custom message
    pub fn message(mut self, msg: &str) -> Self {
        self.message = msg.to_string();
        self
    }

    /// Add paths to skip
    pub fn skip(mut self, paths: Vec<&str>) -> Self {
        self.skip_paths = paths.iter().map(|s| s.to_string()).collect();
        self
    }
}

/// Rate limiter entry for tracking requests
#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

/// Rate limiter state
#[derive(Debug, Clone)]
pub struct RateLimiter {
    config: RateLimiterConfig,
    entries: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a request should be allowed
    pub fn check(&self, key: &str) -> RateLimitResult {
        let mut entries = self.entries.write();
        let now = Instant::now();

        let entry = entries.entry(key.to_string()).or_insert(RateLimitEntry {
            count: 0,
            window_start: now,
        });

        // Reset window if expired
        if now.duration_since(entry.window_start) >= self.config.window {
            entry.count = 0;
            entry.window_start = now;
        }

        entry.count += 1;

        if entry.count > self.config.max_requests {
            let retry_after = self.config.window.as_secs() as u32
                - now.duration_since(entry.window_start).as_secs() as u32;

            RateLimitResult::Exceeded {
                retry_after,
                limit: self.config.max_requests,
                remaining: 0,
            }
        } else {
            RateLimitResult::Allowed {
                limit: self.config.max_requests,
                remaining: self.config.max_requests - entry.count,
            }
        }
    }

    /// Get the config
    pub fn config(&self) -> &RateLimiterConfig {
        &self.config
    }
}

/// Result of rate limit check
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed {
        limit: u32,
        remaining: u32,
    },
    Exceeded {
        retry_after: u32,
        limit: u32,
        remaining: u32,
    },
}

/// Create rate limiting middleware
///
/// # Example
///
/// ```rust,ignore
/// use rustyx::middleware::rate_limit::{rate_limiter, RateLimiterConfig};
///
/// let config = RateLimiterConfig::new(100, 60); // 100 requests per minute
/// app.use_middleware(rate_limiter(config));
/// ```
pub fn rate_limiter(
    config: RateLimiterConfig,
) -> impl Fn(
    Request,
    Response,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Send
       + Sync
       + Clone
       + 'static {
    let limiter = RateLimiter::new(config.clone());

    move |req: Request, res: Response, next: Next| {
        let limiter = limiter.clone();
        let config = config.clone();

        Box::pin(async move {
            // Skip rate limiting for certain paths
            if config.skip_paths.iter().any(|p| req.path().starts_with(p)) {
                return next(req, res).await;
            }

            // Use IP address as the rate limit key
            let key = req.ip().to_string();

            match limiter.check(&key) {
                RateLimitResult::Allowed { limit, remaining } => {
                    let res = res
                        .header("X-RateLimit-Limit", &limit.to_string())
                        .header("X-RateLimit-Remaining", &remaining.to_string());
                    next(req, res).await
                }
                RateLimitResult::Exceeded {
                    retry_after,
                    limit,
                    remaining: _,
                } => res
                    .status(429)
                    .header("X-RateLimit-Limit", &limit.to_string())
                    .header("X-RateLimit-Remaining", "0")
                    .header("Retry-After", &retry_after.to_string())
                    .json(serde_json::json!({
                        "error": "Too Many Requests",
                        "message": config.message,
                        "retry_after": retry_after
                    })),
            }
        })
    }
}

/// Simple rate limiter - shorthand for common use cases
pub fn simple_rate_limit(
    max_requests: u32,
    window_secs: u64,
) -> impl Fn(
    Request,
    Response,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Send
       + Sync
       + Clone
       + 'static {
    rate_limiter(RateLimiterConfig::new(max_requests, window_secs))
}
