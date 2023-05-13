use axum::http::{Request, Response, Uri};
use std::{
    borrow::Cow,
    task::{Context, Poll},
};
use tower::{Layer, Service};

#[derive(Debug, Copy, Clone)]
pub struct NormalizePathLayer {}

impl NormalizePathLayer {
    pub fn trim_trailing_slash() -> Self {
        NormalizePathLayer {}
    }
}

impl<S> Layer<S> for NormalizePathLayer {
    type Service = NormalizePath<S>;

    fn layer(&self, inner: S) -> Self::Service {
        NormalizePath::trim_trailing_slash(inner)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NormalizePath<S> {
    inner: S,
}

impl<S> NormalizePath<S> {
    pub fn trim_trailing_slash(inner: S) -> Self {
        Self { inner }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for NormalizePath<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        normalize_trailing_slash(req.uri_mut());
        self.inner.call(req)
    }
}

fn normalize_trailing_slash(uri: &mut Uri) {
    if !uri.path().ends_with('/') {
        return;
    }

    let new_path = uri.path().trim_end_matches('/');

    let mut parts = uri.clone().into_parts();

    let new_path_and_query = if let Some(path_and_query) = &parts.path_and_query {
        let new_path = if new_path.is_empty() { "/" } else { new_path };

        let new_path_and_query = if let Some(query) = path_and_query.query() {
            Cow::Owned(format!("{}?{}", new_path, query))
        } else {
            new_path.into()
        }
        .parse()
        .unwrap();

        Some(new_path_and_query)
    } else {
        None
    };

    parts.path_and_query = new_path_and_query;
    if let Ok(new_uri) = Uri::from_parts(parts) {
        *uri = new_uri;
    }
}
