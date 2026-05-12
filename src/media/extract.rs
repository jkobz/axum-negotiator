//! Provides **media-type** [Extractor].

use std::marker::PhantomData;
use std::ops::Deref;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::IntoResponse;
use futures::TryFutureExt;

use super::{Either, Supported, Type};
use crate::header::{self};
use crate::media;

/// # **Media-type** extractor.
///
/// Populates **media-type** from `header`,
/// validates it via [Supported].
#[derive(Clone)]
pub struct Extractor<M, E, X = E, H = header::Accept> {
    media: M,
    header: PhantomData<H>,
    extraction_error: PhantomData<X>,
    conversion_error: PhantomData<E>,
}

impl<M, E, X, H> Extractor<M, E, X, H> {
    fn new(media: M) -> Self {
        Self {
            media,
            header: PhantomData,
            extraction_error: PhantomData,
            conversion_error: PhantomData,
        }
    }

    pub fn into_inner(self) -> M {
        self.media
    }
}

impl<M, E, X, H> Deref for Extractor<M, E, X, H> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.media
    }
}

impl<M, E, X, H, S> FromRequestParts<S> for Extractor<M, E, X, H>
where
    M: Supported + for<'a> TryFrom<&'a Type, Error = media::Rejection<M>>,
    E: From<media::Rejection<M>> + IntoResponse,
    X: From<H::Rejection> + IntoResponse,
    H: FromRequestParts<S> + Into<Type> + Send + Sync,
    S: Send + Sync,
{
    type Rejection = Either<E, X>;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        H::from_request_parts(parts, state)
            .map_err(|err| Either::Second(err.into()))
            .and_then(|header| async {
                M::try_from(&header.into())
                    .map(Self::new)
                    .map_err(|err| Either::First(err.into()))
            })
            .await
    }
}
