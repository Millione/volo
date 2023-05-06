use futures::Future;
use volo::Service;

use crate::{body::Body, context::ServerContext, Request, Response, Status};

#[derive(Clone, Debug)]
pub struct ConvertService<S> {
    inner: S,
}

impl<S> ConvertService<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> Service<ServerContext, hyper::Request<hyper::Body>> for ConvertService<S>
where
    S: Service<ServerContext, Request<hyper::Body>, Response = Response<Body>>
        + Clone
        + Send
        + Sync
        + 'static,
    S::Error: Into<Status>,
{
    type Response = hyper::Response<Body>;

    type Error = Status;

    type Future<'cx> = impl Future<Output = Result<Self::Response, Self::Error>> + Send + 'cx;

    fn call<'cx, 's>(
        &'s self,
        cx: &'cx mut ServerContext,
        req: hyper::Request<hyper::Body>,
    ) -> Self::Future<'cx>
    where
        's: 'cx,
    {
        async move {
            let volo_req = Request::from_http(req);

            let resp = match self.inner.call(cx, volo_req).await {
                Ok(resp) => resp,
                Err(err) => {
                    return Ok(err.into().to_http());
                }
            };

            let mut resp = resp.into_http();

            resp.headers_mut().insert(
                http::header::CONTENT_TYPE,
                http::header::HeaderValue::from_static("application/grpc"),
            );

            Ok(resp)
        }
    }
}
