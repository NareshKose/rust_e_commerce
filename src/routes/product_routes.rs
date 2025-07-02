use actix_web::web;
use crate::handlers::product_handler;
use crate::middleware::auth::AuthMiddleware;

pub  fn product_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/products")
         .wrap(AuthMiddleware)
            .route("", web::post().to(product_handler::add_product))
              .route("", web::get().to(product_handler::get_products)) // all products (listed/unlisted)
            .route("/{id}", web::get().to(product_handler::get_product_by_id))
            .route("/category/{category}", web::get().to(product_handler::get_products_by_category))
            .route("/{id}/status", web::put().to(product_handler::update_product_status)) // update status route
             .route("/{id}", web::put().to(product_handler::update_product)) 
    );
}
