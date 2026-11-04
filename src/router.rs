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
#[derive(Clone)]
pub struct RouteHandler {
    handler: HandlerFn,
}

/// Express-like Router for grouping routes
#[derive(Clone)]
pub struct Router {
    routes: HashMap<Method, MatchitRouter<RouteHandler>>,
    prefix: String,
    entries: Vec<(Method, String, HandlerFn)>,
}

impl Router {
    /// Create a new Router instance
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            prefix: String::new(),
            entries: Vec::new(),
        }
    }

    /// Create a new Router with a path prefix
    pub fn with_prefix(prefix: &str) -> Self {
        Self {
            routes: HashMap::new(),
            prefix: prefix.to_string(),
            entries: Vec::new(),
        }
    }

    /// Add a route to the router
    pub fn add_route(&mut self, method: Method, path: &str, handler: HandlerFn) {
        let full_path = format!("{}{}", self.prefix, path);
        let router = self.routes.entry(method.clone()).or_insert_with(MatchitRouter::new);

        if let Err(e) = router.insert(&full_path, RouteHandler { handler: Arc::clone(&handler) }) {
            tracing::warn!("Failed to insert route {}: {:?}", full_path, e);
        }

        self.entries.push((method, full_path, handler));
    }

    /// Find a route handler for the given method and path
    pub fn find_route(
        &self,
        method: &Method,
        path: &str,
    ) -> Option<(&HandlerFn, HashMap<String, String>)> {
        if let Some(router) = self.routes.get(method) {
            if let Ok(matched) = router.at(path) {
                let mut params = HashMap::with_capacity(matched.params.len());
                for (k, v) in matched.params.iter() {
                    params.insert(k.to_string(), v.to_string());
                }
                return Some((&matched.value.handler, params));
            }
        }
        None
    }

    /// Mount another router at a path prefix
    pub fn mount(&mut self, prefix: &str, other: Router) {
        let clean_prefix = prefix.trim_end_matches('/');
        for (method, path, handler) in other.entries {
            self.add_route(method, &format!("{}{}", clean_prefix, path), handler);
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
        let mut group_router = Router::new();
        configure(&mut group_router);
        self.mount(prefix, group_router);
        self
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_routing() {
        let mut router = Router::new();
        router.add_route(
            Method::GET,
            "/users/:id",
            Arc::new(|_req, res| Box::pin(async move { res })),
        );

        let (handler, params) = router.find_route(&Method::GET, "/users/123").expect("Route should match");
        assert_eq!(params.get("id"), Some(&"123".to_string()));
        let _ = handler;
    }

    #[test]
    fn test_group_routing() {
        let mut router = Router::new();
        router.group("/api", |api| {
            api.add_route(
                Method::GET,
                "/users",
                Arc::new(|_req, res| Box::pin(async move { res })),
            );
        });

        let found = router.find_route(&Method::GET, "/api/users");
        assert!(found.is_some());
    }
}
