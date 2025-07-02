use actix_web::web;
use crate::handlers::auth_handler::{signup_user, login_user, logout_user};

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/signup", web::post().to(signup_user))
        .route("/login", web::post().to(login_user))
        .route("/logout", web::post().to(logout_user));
}
