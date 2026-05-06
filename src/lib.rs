#![doc = include_str!("../README.md")]

pub mod error;
pub mod header;
pub mod media;

use std::marker::PhantomData;
use std::mem;
use std::task::{Context, Poll};

use axum_core::RequestExt;
use axum_core::extract::{FromRequestParts, Request};
use axum_core::response::{IntoResponse, Response};
use axum_extra::either::Either;
pub use error::Rejection;
use futures::future::{BoxFuture, TryFutureExt};

/// # Content-Negotiation trait.
///
/// Provides [into_response] method to turn `self`
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

/// # Content-Negotiation **middleware**.
///
/// See [Service].
#[derive(Clone)]
pub struct Layer<M, E, S> {
    state: S,
    supported_media: PhantomData<M>,
    error: PhantomData<E>,
}

/// # Fallback-state.
///
/// Used to content-negotiate [Result::Err] via `fallback`.
#[derive(Clone)]
pub struct WithFallback<F> {
    /// Fallback **media-type**.
    fallback: F,
}

impl<M, E> Layer<M, E, ()> {
    pub fn new() -> Self {
        Self {
            state: (),
            supported_media: PhantomData,
            error: PhantomData,
        }
    }
}

impl<M, E, F> Layer<M, E, WithFallback<F>> {
    pub fn with_fallback(fallback: F) -> Self {
        Self {
            state: WithFallback { fallback },
            supported_media: PhantomData,
            error: PhantomData,
        }
    }
}

impl<M, E, F> Default for Layer<M, E, WithFallback<F>>
where
    F: Default,
{
    fn default() -> Self {
        Self {
            state: WithFallback {
                fallback: F::default(),
            },
            supported_media: PhantomData,
            error: PhantomData,
        }
    }
}

impl<I, M, E, S> tower_layer::Layer<I> for Layer<M, E, S>
where
    I: Clone + Send + Sync,
    S: Clone,
{
    type Service = Service<I, M, E, S>;

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
/// throws `error` otherwise.
///
/// Content-negotiates response [Result] value via [Negotiate],
/// erasing both [Result::Ok] and [Result::Err] types.
#[derive(Clone)]
pub struct Service<I, M, E, S> {
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

    /// # Error type.
    ///
    /// Indicates what [Rejection] should be coerced to.
    error: PhantomData<E>,
}

impl<I, M, E, S> Service<I, M, E, S>
where
    S: Clone,
{
    pub fn new(inner: I, state: &S) -> Self {
        Self {
            state: state.clone(),
            inner,
            supported_media: PhantomData,
            error: PhantomData,
        }
    }
}

impl<R, I, M, E> tower_service::Service<R> for Service<I, M, E, ()>
where
    R: Into<Request> + Send + 'static,
    I: tower_service::Service<Request> + Send + Sync + Clone + 'static,
    I::Future: Send + 'static,
    I::Response: Send + Negotiate<M>,
    I::Error: Send + Negotiate<M>,
    M: media::Stateful<I::Response> + Send + Sync + 'static,
    media::Extractor<M, E>:
        FromRequestParts<(), Rejection = E> + Send + Sync + 'static,
{
    type Error = Either<I::Error, E>;
    type Future = BoxFuture<'static, Result<Response, Self::Error>>;
    type Response = Response;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Either::E1)
    }

    fn call(&mut self, req: R) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = mem::replace(&mut self.inner, clone);
        let mut req = req.into();
        Box::pin(async move {
            let media = req
                .extract_parts::<media::Extractor<M, E>>()
                .map_err(Either::E2)
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

impl<R, I, M, E, F> tower_service::Service<R>
    for Service<I, M, E, WithFallback<F>>
where
    R: Into<Request> + Send + 'static,
    I: tower_service::Service<Request> + Send + Clone + 'static,
    I::Future: Send + 'static,
    I::Response: Send + Negotiate<M>,
    I::Error: Send + Negotiate<F>,
    M: media::Stateful<I::Response> + Send + Sync + 'static,
    media::Extractor<M, E>:
        FromRequestParts<(), Rejection = E> + Send + Sync + 'static,
    F: Clone + Send + Sync + 'static,
{
    type Error = Either<I::Error, E>;
    type Future = BoxFuture<'static, Result<Response, Self::Error>>;
    type Response = Response;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Either::E1)
    }

    fn call(&mut self, req: R) -> Self::Future {
        let fallback = self.state.fallback.clone();
        let clone = self.inner.clone();
        let mut inner = mem::replace(&mut self.inner, clone);
        let mut req = req.into();
        Box::pin(async move {
            let media = req
                .extract_parts::<media::Extractor<M, E>>()
                .map_err(Either::E2)
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
