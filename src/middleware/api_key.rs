use crate::api::handler::GenericResponse;
use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::future::{ok, Either, Ready};
use std::task::{Context, Poll};

const API_KEY: &str = "your_secret_api_key";

// Middleware pour vérifier l'API key
pub struct ApiKey;

impl<S> Transform<S, ServiceRequest> for ApiKey
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = ApiKeyMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ApiKeyMiddleware { service })
    }
}

pub struct ApiKeyMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for ApiKeyMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if req.uri().path().starts_with("/ws/") {
            return Either::Left(self.service.call(req));
        }

        if let Some(key) = req.headers().get("X-Api-Key") {
            if key == API_KEY {
                return Either::Left(self.service.call(req));
            }
        }

        let response = GenericResponse {
            status: "error".to_string(),
            message: "Unauthorized".to_string(),
            value: vec![{}].into(),
        };

        let response = HttpResponse::Unauthorized().json(response);
        Either::Right(ok(req.into_response(response.map_into_boxed_body())))
    }
}
