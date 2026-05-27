#![doc = include_str!("../README.md")]

pub mod header;
pub mod media;
pub mod payload;

use std::marker::PhantomData;
use std::convert::Infallible;
use std::mem;
use std::task::{Context, Poll};

use axum::RequestExt;
use axum::extract::{FromRequestParts, Request};
use axum::response::{IntoResponse, Response, Redirect};
use futures::future::{BoxFuture, TryFutureExt};
use media::Either;
pub use payload::Payload;

/// # Content-negotiation trait.
///
/// Provides [Negotiate::into_response] method to turn `self`
/// into [Response] via *generic* `media` **media-type**.
///
/// Provides *blanket* impl for [header::Status] implementors.
pub trait Negotiate<M> {
    fn into_response(&self, media: &M) -> Response;
}

impl<T, M> Negotiate<M> for T
where
    T: header::Status,
    M: media::Stateful<T>,
{
    fn into_response(&self, media: &M) -> Response {
        (self.status(), media.with(self)).into_response()
    }
}

impl Negotiate<media::Html> for Redirect {
    fn into_response(&self, _: &media::Html) -> Response {
        self.clone().into_response()
    }
}

impl<M> Negotiate<M> for Infallible {
    fn into_response(&self, _: &M) -> Response {
        match *self {}
    }
}

/// # Content-negotiation **middleware**.
///
/// See [Service].
#[derive(Clone)]
pub struct Layer<M, E, X, S> {
    state: S,
    supported_media: PhantomData<M>,
    conversion_error: PhantomData<E>,
    extraction_error: PhantomData<X>,
}

/// # Fallback-state.
///
/// Used to content-negotiate [Result::Err] via `fallback`.
#[derive(Clone)]
pub struct WithFallback<F> {
    /// Fallback **media-type**.
    fallback: F,
}

impl<M, E, X> Layer<M, E, X, ()> {
    pub fn new() -> Self {
        Self {
            state: (),
            supported_media: PhantomData,
            conversion_error: PhantomData,
            extraction_error: PhantomData,
        }
    }
}

impl<M, E, X, F> Layer<M, E, X, WithFallback<F>> {
    pub fn with_fallback(fallback: F) -> Self {
        Self {
            state: WithFallback { fallback },
            supported_media: PhantomData,
            conversion_error: PhantomData,
            extraction_error: PhantomData,
        }
    }
}

impl<M, E, X, F: Default> Default for Layer<M, E, X, WithFallback<F>> {
    fn default() -> Self {
        Self::with_fallback(F::default())
    }
}

impl<I, M, E, X, S> tower_layer::Layer<I> for Layer<M, E, X, S>
where
    I: Clone + Send + Sync,
    S: Clone,
{
    type Service = Service<I, M, E, X, S>;

    fn layer(&self, inner: I) -> Self::Service {
        Service::new(inner, &self.state)
    }
}

/// # Content-negotiation **service**.
/// Accepted media-type **service**.
///
/// Serializes `inner` wrapped middleware **response**
/// [Result] value to the *accepted* **media-type**.
///
/// Validates whether the **media-type** is `supported_media`,
/// throws `conversion_error` otherwise.
///
/// Content-negotiates response [Result] value via [Negotiate],
/// erasing both [Result::Ok] and [Result::Err] types.
#[derive(Clone)]
pub struct Service<I, M, E, X, S> {
    /// Arbitrary state.
    state: S,

    /// Wrapped middleware.
    inner: I,

    /// # Supported **media-type**.
    ///
    /// Can be an `enum` or a `struct`:
    /// ```
    /// enum Media { Html, Json }
    /// ```
    supported_media: PhantomData<M>,

    /// # Conversion error wrapper.
    ///
    /// Used to handle **media-type** conversion error
    /// i.e. a supplied **MIME** is not `supported_media`.
    conversion_error: PhantomData<E>,

    /// # Extraction error wrapper.
    ///
    /// Used to handle **media-type** extraction error
    /// in case of [media::Extractor] rejection.
    extraction_error: PhantomData<X>,
}

impl<I, M, E, X, S: Clone> Service<I, M, E, X, S> {
    pub fn new(inner: I, state: &S) -> Self {
        Self {
            state: state.clone(),
            inner,
            supported_media: PhantomData,
            conversion_error: PhantomData,
            extraction_error: PhantomData,
        }
    }
}

impl<R, I, M, E, X> tower_service::Service<R> for Service<I, M, E, X, ()>
where
    R: Into<Request> + Send + 'static,
    I: tower_service::Service<Request> + Send + Sync + Clone + 'static,
    I::Future: Send + 'static,
    I::Response: Send + Negotiate<M>,
    I::Error: Send + Negotiate<M>,
    media::Extractor<M, E, X>:
        FromRequestParts<(), Rejection = Either<E, X>> + Send + Sync + 'static,
{
    type Error = Either<I::Error, Either<E, X>>;
    type Future = BoxFuture<'static, Result<Response, Self::Error>>;
    type Response = Response;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Either::First)
    }

    fn call(&mut self, req: R) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = mem::replace(&mut self.inner, clone);
        let mut req = req.into();
        Box::pin(async move {
            let media = req
                .extract_parts::<media::Extractor<M, E, X>>()
                .map_err(Either::Second)
                .await?;
            inner
                .call(req)
                .map_ok_or_else(
                    |err| Ok(err.into_response(&media)),
                    |res| Ok(res.into_response(&media)),
                )
                .await
        })
    }
}

impl<R, I, M, E, X, F> tower_service::Service<R>
    for Service<I, M, E, X, WithFallback<F>>
where
    R: Into<Request> + Send + 'static,
    I: tower_service::Service<Request> + Send + Clone + 'static,
    I::Future: Send + 'static,
    I::Response: Send + Negotiate<M>,
    I::Error: Send + Negotiate<F>,
    media::Extractor<M, E, X>:
        FromRequestParts<(), Rejection = Either<E, X>> + Send + Sync + 'static,
    F: Clone + Send + Sync + 'static,
{
    type Error = Either<I::Error, Either<E, X>>;
    type Future = BoxFuture<'static, Result<Response, Self::Error>>;
    type Response = Response;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Either::First)
    }

    fn call(&mut self, req: R) -> Self::Future {
        let fallback = self.state.fallback.clone();
        let clone = self.inner.clone();
        let mut inner = mem::replace(&mut self.inner, clone);
        let mut req = req.into();
        Box::pin(async move {
            let media = req
                .extract_parts::<media::Extractor<M, E, X>>()
                .map_err(Either::Second)
                .await?;
            inner
                .call(req)
                .map_ok_or_else(
                    |err| Ok(err.into_response(&fallback)),
                    |res| Ok(res.into_response(&media)),
                )
                .await
        })
    }
}
