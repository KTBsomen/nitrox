//! Product Catalog Routes Module

use crate::controllers::ProductController;
use crate::services::AppState;
use rustyx::Router;
use std::sync::Arc;

pub fn router(state: Arc<AppState>) -> Router {
    let mut r = Router::new();
    let controller = ProductController::new(state);

    // List & search products (with query parameters)
    let c = controller.clone();
    r.get("", move |req, res| {
        let c = c.clone();
        async move { c.list_products(req, res).await }
    });
    let c = controller.clone();
    r.get("/", move |req, res| {
        let c = c.clone();
        async move { c.list_products(req, res).await }
    });

    // Create new product
    let c = controller.clone();
    r.post("", move |req, res| {
        let c = c.clone();
        async move { c.create_product(req, res).await }
    });
    let c = controller.clone();
    r.post("/", move |req, res| {
        let c = c.clone();
        async move { c.create_product(req, res).await }
    });

    // Get product by ID
    let c = controller.clone();
    r.get("/:id", move |req, res| {
        let c = c.clone();
        async move { c.get_product(req, res).await }
    });

    // Update product
    let c = controller.clone();
    r.put("/:id", move |req, res| {
        let c = c.clone();
        async move { c.update_product(req, res).await }
    });

    // Delete product
    let c = controller.clone();
    r.delete("/:id", move |req, res| {
        let c = c.clone();
        async move { c.delete_product(req, res).await }
    });

    r
}
