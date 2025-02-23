use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use futures::FutureExt;

pub struct LogMiddleware;

impl<S, B> Transform<S, ServiceRequest> for LogMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static + std::fmt::Debug,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = LogMiddlewareImpl<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LogMiddlewareImpl { service })
    }
}

pub struct LogMiddlewareImpl<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for LogMiddlewareImpl<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: std::fmt::Debug + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        log::info!("Incoming request: {} {}", req.method(), req.uri());

        let fut = self.service.call(req);
        async move {
            let res = fut.await?;
            log::info!("Outgoing response: status {}", res.status());
            Ok(res)
        }
        .boxed_local()
    }
}
