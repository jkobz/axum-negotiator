//! Provides *generic* **request body** extractor [Payload].

use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use axum::RequestExt;
use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::response::IntoResponse;
use axum_extra::either::Either;
use futures::TryFutureExt;

use crate::{header, media};

/// # *Generic* **HTTP** request **body** payload.
///
/// Implements [FromRequest] that extracts the
/// *expected* request **body** **content-type**,
/// rejects if *unsupported*, deserializes the value.
pub struct Payload<T, M, E, X> {
    /// Deserialized value.
    value: T,

    /// # *Supported* payload **media-type**.
    ///
    /// Extracted via [CONTENT_TYPE] request header.
    /// Used to negotiate **value** deserialization.
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

impl<T, M, E, X> Payload<T, M, E, X> {
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T, M, E, X> Payload<T, M, E, X> {
    fn new(value: T) -> Self {
        Self {
            value,
            supported_media: PhantomData,
            conversion_error: PhantomData,
            extraction_error: PhantomData,
        }
    }
}

impl<T, M, E, X> Deref for Payload<T, M, E, X> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, M, E, X> DerefMut for Payload<T, M, E, X> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T, M, E, X> fmt::Debug for Payload<T, M, E, X>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

impl<S, T, M, E, X> FromRequest<S> for Payload<T, M, E, X>
where
    S: Send + Sync,
    T: Send + 'static,
    M: Extract<T> + media::Supported + Send + Sync + 'static,
    E: From<media::Rejection<M>> + IntoResponse + 'static,
    X: From<<header::ContentType as FromRequestParts<()>>::Rejection>
        + IntoResponse
        + 'static,
    media::Extractor<M, E, X, header::ContentType>: FromRequestParts<(), Rejection = media::extract::Rejection<E, X>>
        + Send
        + Sync
        + 'static,
    <M as Extract<T>>::Rejection: IntoResponse,
{
    type Rejection =
        Either<media::extract::Rejection<E, X>, <M as Extract<T>>::Rejection>;

    async fn from_request(
        mut req: Request,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let media = req
            .extract_parts::<media::Extractor<M, E, X, header::ContentType>>()
            .map_err(Either::E1)
            .await?;
        media
            .extract(req)
            .map_err(Either::E2)
            .map_ok(Self::new)
            .await
    }
}

/// Provides [extract] method to consume the **HTTP** request
/// into an arbitrary `T` value, given the implementor **media-type**.
pub trait Extract<T> {
    type Rejection;

    fn extract(
        &self,
        req: Request,
    ) -> impl Future<Output = Result<T, Self::Rejection>> + Send;
}
