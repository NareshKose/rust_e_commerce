use actix_web::web;
use crate::handlers::order_handler;
use crate::middleware::auth::AuthMiddleware;

pub fn order_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/orders")
            .wrap(AuthMiddleware)
            .route("/place", web::post().to(order_handler::place_order))
            .route("/{order_id}/status", web::patch().to(order_handler::update_order_status))
            .route("/{order_id}/details", web::patch().to(order_handler::get_order_details))
            .route("/{order_id}/cancel", web::post().to(order_handler::cancel_order))
    );
}
