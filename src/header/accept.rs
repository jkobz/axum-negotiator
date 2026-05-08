//! Provides [Accept] **HTTP** header.

use std::cmp::Ordering;
use std::fmt;
use std::iter::once;
use std::ops::Deref;
use std::str::FromStr;

use axum::extract::FromRequestParts;
use axum::http::header::{ACCEPT, HeaderName, HeaderValue};
use axum::http::request::Parts;
use axum_extra::headers::{Error, Header};
use axum_extra::typed_header::{TypedHeader, TypedHeaderRejection};
use futures::TryFutureExt;
use mime::STAR_STAR;

use crate::media;

/// # **Accept** **HTTP** request header.
pub struct Accept(media::Type);

const QUALITY_KEY: &str = "q=";
const QUALITY_VALUE: f32 = 1.0;

impl Accept {
    /// Attempts to parse an *arbitrary* `s` into
    /// the **media-type** with **quality value**.
    ///
    /// Defaults **quality** `q` param to **1.0**.
    fn parse_item(s: &str) -> Option<(media::Type, f32)> {
        let mut parts = s.split(";");
        let mime = parts.next()?.trim();
        let mime = media::Type::from_str(mime).ok()?;
        let mut quality = QUALITY_VALUE;
        for param in parts {
            if let Some(v) = param.trim().strip_prefix(QUALITY_KEY) {
                if let Ok(parsed) = v.parse::<f32>() {
                    quality = parsed.clamp(0.0, 1.0);
                }
            }
        }
        Some((mime, quality))
    }
}

impl From<media::Type> for Accept {
    fn from(value: media::Type) -> Self {
        Self(value)
    }
}

impl From<Accept> for media::Type {
    fn from(value: Accept) -> Self {
        value.0
    }
}

impl Deref for Accept {
    type Target = media::Type;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Accept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Header for Accept {
    fn name() -> &'static HeaderName {
        &ACCEPT
    }

    fn decode<'a, I>(values: &mut I) -> Result<Self, Error>
    where
        I: Iterator<Item = &'a HeaderValue>,
    {
        let value = values
            .next()
            .and_then(|v| v.to_str().ok())
            .ok_or_else(Error::invalid)?;
        let best = value
            .split(",")
            .filter_map(Self::parse_item)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
            .map(|(mime, _)| mime)
            .ok_or_else(Error::invalid)?;
        Ok(Self(best))
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

impl<S> FromRequestParts<S> for Accept
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
