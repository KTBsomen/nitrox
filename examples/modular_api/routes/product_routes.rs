//! Product Catalog Routes Module

use crate::controllers::ProductController;
use crate::services::AppState;
use nitrox::Router;

pub fn router() -> Router<AppState> {
    let mut r = Router::with_state();

    // List & search products (with query parameters)
    r.get("", |req, res| async move {
        ProductController::list_products(req, res).await
    });
    r.get("/", |req, res| async move {
        ProductController::list_products(req, res).await
    });

    // Create new product
    r.post("", |req, res| async move {
        ProductController::create_product(req, res).await
    });
    r.post("/", |req, res| async move {
        ProductController::create_product(req, res).await
    });

    // Get product by ID
    r.get("/:id", |req, res| async move {
        ProductController::get_product(req, res).await
    });

    // Update product
    r.put("/:id", |req, res| async move {
        ProductController::update_product(req, res).await
    });

    // Delete product
    r.delete("/:id", |req, res| async move {
        ProductController::delete_product(req, res).await
    });

    r
}
