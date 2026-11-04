//! Router Module
//!
//! Provides routing functionality similar to Express Router.

use crate::app::HandlerFn;
use crate::request::Request;
use crate::response::Response;

use hyper::Method;
use matchit::Router as MatchitRouter;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

/// Route handler with its matched parameters
pub struct RouteHandler {
    handler: HandlerFn,
}

/// Express-like Router for grouping routes
pub struct Router {
    routes: HashMap<Method, MatchitRouter<RouteHandler>>,
    prefix: String,
}

impl Router {
    /// Create a new Router instance
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            prefix: String::new(),
        }
    }

    /// Create a new Router with a path prefix
    pub fn with_prefix(prefix: &str) -> Self {
        Self {
            routes: HashMap::new(),
            prefix: prefix.to_string(),
        }
    }

    /// Add a route to the router
    pub fn add_route(&mut self, method: Method, path: &str, handler: HandlerFn) {
        let full_path = format!("{}{}", self.prefix, path);
        let router = self.routes.entry(method).or_insert_with(MatchitRouter::new);

        // Convert Express-style params (:id) to matchit style ({id})
        let converted_path = convert_express_params(&full_path);

        if let Err(e) = router.insert(&converted_path, RouteHandler { handler }) {
            tracing::warn!("Failed to insert route {}: {:?}", converted_path, e);
        }
    }

    /// Find a route handler for the given method and path
    pub fn find_route(
        &self,
        method: &Method,
        path: &str,
    ) -> Option<(&HandlerFn, HashMap<String, String>)> {
        if let Some(router) = self.routes.get(method) {
            if let Ok(matched) = router.at(path) {
                let params: HashMap<String, String> = matched
                    .params
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                return Some((&matched.value.handler, params));
            }
        }
        None
    }

    /// Mount another router at a path prefix
    pub fn mount(&mut self, _prefix: &str, mut other: Router) {
        for (method, other_router) in other.routes.drain() {
            self.routes.entry(method).or_insert(other_router);
        }
    }

    /// Register a GET route
    pub fn get<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add_route(
            Method::GET,
            path,
            Arc::new(move |req, res| Box::pin(handler(req, res))),
        );
        self
    }

    /// Register a POST route
    pub fn post<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add_route(
            Method::POST,
            path,
            Arc::new(move |req, res| Box::pin(handler(req, res))),
        );
        self
    }

    /// Register a PUT route
    pub fn put<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add_route(
            Method::PUT,
            path,
            Arc::new(move |req, res| Box::pin(handler(req, res))),
        );
        self
    }

    /// Register a DELETE route
    pub fn delete<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add_route(
            Method::DELETE,
            path,
            Arc::new(move |req, res| Box::pin(handler(req, res))),
        );
        self
    }

    /// Register a PATCH route
    pub fn patch<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add_route(
            Method::PATCH,
            path,
            Arc::new(move |req, res| Box::pin(handler(req, res))),
        );
        self
    }

    /// Create a route group with a common prefix
    pub fn group<F>(&mut self, prefix: &str, configure: F) -> &mut Self
    where
        F: FnOnce(&mut Router),
    {
        let mut group_router = Router::with_prefix(&format!("{}{}", self.prefix, prefix));
        configure(&mut group_router);

        // Merge group routes into main router
        for (method, group_routes) in group_router.routes {
            self.routes.entry(method).or_insert(group_routes);
        }

        self
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert Express-style route parameters to matchit format
fn convert_express_params(path: &str) -> String {
    let mut result = String::with_capacity(path.len());
    let mut chars = path.chars().peekable();

    while let Some(c) = chars.next() {
        if c == ':' {
            result.push('{');
            while let Some(&next) = chars.peek() {
                if next.is_alphanumeric() || next == '_' {
                    result.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            result.push('}');
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_express_params() {
        assert_eq!(convert_express_params("/users/:id"), "/users/{id}");
        assert_eq!(
            convert_express_params("/users/:id/posts/:postId"),
            "/users/{id}/posts/{postId}"
        );
        assert_eq!(convert_express_params("/static"), "/static");
    }
}
