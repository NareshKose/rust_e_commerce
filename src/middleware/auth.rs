use actix_web::{
    body::{BoxBody, EitherBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, HttpMessage,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;
use std::task::{Context, Poll};
use actix_web::cookie::Cookie; // Import Cookie
use crate::utils::jwt::decode_jwt;
use crate::models::auth::Claims;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AuthMiddlewareImpl<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareImpl {
            service: Rc::new(service),
        })
    }
}

pub struct AuthMiddlewareImpl<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareImpl<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Extract cookies from the request
            let cookie_opt = req.cookie("auth_token");

            if let Some(cookie) = cookie_opt {
                let token = cookie.value();

                match decode_jwt(&token) {
                    Ok(claims) => {
                        req.extensions_mut().insert(claims);
                        service.call(req).await.map(|res| res.map_into_left_body())
                    }
                    Err(_) => {
                        let res = HttpResponse::Unauthorized()
                            .body("Invalid token");
                        Ok(req.into_response(res).map_into_right_body())
                    }
                }
            } else {
                let res = HttpResponse::Unauthorized()
                    .body("Authentication token missing");
                Ok(req.into_response(res).map_into_right_body())
            }
        })
    }
}