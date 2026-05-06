//! Provides **media-type** [Extractor].

use std::marker::PhantomData;
use std::ops::Deref;

use axum_core::extract::FromRequestParts;
use axum_core::response::IntoResponse;
use futures::TryFutureExt;
use http::request::Parts;

use super::{Supported, Type};
use crate::{Rejection, header};

/// # **Media-type** extractor.
///
/// Populates **media-type** from `header`,
/// validates it via [Supported].
#[derive(Clone)]
pub struct Extractor<M, E, H = header::Accept> {
    media: M,
    header: PhantomData<H>,
    error: PhantomData<E>,
}

impl<M, E, H> Extractor<M, E, H> {
    fn new(media: M) -> Self {
        Self {
            media,
            header: PhantomData,
            error: PhantomData,
        }
    }

    pub fn into_inner(self) -> M {
        self.media
    }
}

impl<M, E, H> Deref for Extractor<M, E, H> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.media
    }
}

impl<M, E, H, S> FromRequestParts<S> for Extractor<M, E, H>
where
    M: Supported + TryFrom<Type, Error = Rejection<M>>,
    E: From<Rejection<M>> + From<H::Rejection> + IntoResponse,
    H: FromRequestParts<S> + Into<Type> + Send + Sync,
    S: Send + Sync,
{
    type Rejection = E;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        H::from_request_parts(parts, state)
            .err_into()
            .and_then(|header| async {
                M::try_from(header.into())
                    .map(Self::new)
                    .map_err(Into::into)
            })
            .await
    }
}
