use actix_web::web;
use crate::handlers::report_handler;
use crate::middleware::auth::AuthMiddleware;

pub fn report_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/reports")
            .wrap(AuthMiddleware)
            .route("/{report_type}", web::get().to(report_handler::fetch_report_by_type))
    );
}
