use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::body::{Bytes, HttpBody};
use axum::http::request::Request;
use axum::response::Response;
use tower::Service;

use crate::vite_serve::ViteServe;

impl<B> Service<Request<B>> for ViteServe
where
    B: HttpBody<Data = Bytes> + Send + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let svc = self.clone();
        Box::pin(async move { Ok(svc.serve(req).await) })
    }
}
