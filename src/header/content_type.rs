//! Provides [ContentType] **HTTP** header.

use std::fmt;
use std::iter::once;
use std::ops::Deref;

use axum::extract::FromRequestParts;
use axum::http::header::{CONTENT_TYPE, HeaderName, HeaderValue};
use axum::http::request::Parts;
use axum_extra::headers::{Error, Header};
use axum_extra::typed_header::{TypedHeader, TypedHeaderRejection};
use futures::TryFutureExt;
use mime::STAR_STAR;

use crate::media;

/// **Content-Type** **HTTP** request header.
pub struct ContentType(media::Type);

impl From<media::Type> for ContentType {
    fn from(value: media::Type) -> Self {
        Self(value)
    }
}

impl From<ContentType> for media::Type {
    fn from(value: ContentType) -> Self {
        value.0
    }
}

impl Deref for ContentType {
    type Target = media::Type;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Header for ContentType {
    fn name() -> &'static HeaderName {
        &CONTENT_TYPE
    }

    fn decode<'a, I>(values: &mut I) -> Result<Self, Error>
    where
        I: Iterator<Item = &'a HeaderValue>,
    {
        values
            .next()
            .ok_or_else(Error::invalid)
            .and_then(|v| v.to_str().map_err(|_| Error::invalid()))
            .and_then(|s| {
                s.parse::<media::Type>().map_err(|_| Error::invalid())
            })
            .map(Self)
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let header =
            HeaderValue::from_str(self.as_ref()).unwrap_or_else(|_| {
                HeaderValue::from_static(STAR_STAR.essence_str())
            });
        values.extend(once(header));
    }
}

impl<S> FromRequestParts<S> for ContentType
where
    S: Send + Sync,
{
    type Rejection = TypedHeaderRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        TypedHeader::<Self>::from_request_parts(parts, state)
            .map_ok(|h| h.0)
            .await
    }
}
