//! Product Controller

use crate::models::{CreateProductDto, ProductFilterQuery, UpdateProductDto};
use crate::services::AppState;
use crate::utils::{created, ok, paginated};
use nitrox::{Request, Response};

pub struct ProductController;

impl ProductController {
    pub async fn list_products(req: Request<AppState>, res: Response) -> Response {
        let page: u32 = req
            .query_param("page")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        let limit: u32 = req
            .query_param("limit")
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);
        let search = req.query_param("q").and_then(|v| v.parse().ok());
        let category = req.query_param("category").and_then(|v| v.parse().ok());
        let min_price = req.query_param("min_price").and_then(|v| v.parse().ok());
        let max_price = req.query_param("max_price").and_then(|v| v.parse().ok());

        let filter = ProductFilterQuery {
            search,
            category,
            min_price,
            max_price,
            page,
            limit,
        };

        let (products, total) = req.state.product_service.query(filter).await;
        paginated(res, products, total, page, limit)
    }

    pub async fn get_product(req: Request<AppState>, res: Response) -> Response {
        let id = req.param("id").cloned().unwrap_or_default();
        match req.state.product_service.get_by_id(&id).await {
            Ok(product) => ok(res, product),
            Err(err) => err.to_response(res),
        }
    }

    pub async fn create_product(req: Request<AppState>, res: Response) -> Response {
        match req.json::<CreateProductDto>() {
            Ok(dto) => match req.state.product_service.create(dto).await {
                Ok(product) => created(res, "Product created successfully", product),
                Err(err) => err.to_response(res),
            },
            Err(e) => res.bad_request(&format!("Invalid JSON body: {}", e)),
        }
    }

    pub async fn update_product(req: Request<AppState>, res: Response) -> Response {
        let id = req.param("id").cloned().unwrap_or_default();
        match req.json::<UpdateProductDto>() {
            Ok(dto) => match req.state.product_service.update(&id, dto).await {
                Ok(product) => ok(res, product),
                Err(err) => err.to_response(res),
            },
            Err(e) => res.bad_request(&format!("Invalid JSON body: {}", e)),
        }
    }

    pub async fn delete_product(req: Request<AppState>, res: Response) -> Response {
        let id = req.param("id").cloned().unwrap_or_default();
        match req.state.product_service.delete(&id).await {
            Ok(_) => ok(
                res,
                serde_json::json!({ "message": "Product deleted", "id": id }),
            ),
            Err(err) => err.to_response(res),
        }
    }
}
