//! Provides [Either] **media-type**.

use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use futures::TryFutureExt;
use serde::{Serialize, Serializer};

use super::{Rejection, Stateful, Supported, Type};
use crate::{Negotiate, payload};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Either<M, N> {
    First(M),
    Second(N),
}

impl<M, A, B> Negotiate<M> for Either<A, B>
where
    A: Negotiate<M>,
    B: Negotiate<M>,
{
    fn into_response(&self, media: &M) -> Response {
        match self {
            Either::First(a) => a.into_response(media),
            Either::Second(b) => b.into_response(media),
        }
    }
}

impl<M, N> Serialize for Either<M, N>
where
    M: Serialize,
    N: Serialize,
{
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match self {
            Self::First(f) => f.serialize(serializer),
            Self::Second(s) => s.serialize(serializer),
        }
    }
}

impl<M: Default, N> Default for Either<M, N> {
    fn default() -> Self {
        Either::First(M::default())
    }
}

impl<'a, M, N> TryFrom<&'a Type> for Either<M, N>
where
    M: TryFrom<&'a Type> + Supported,
    N: TryFrom<&'a Type> + Supported,
{
    type Error = Rejection<Self>;

    fn try_from(value: &'a Type) -> Result<Self, Self::Error> {
        M::try_from(value)
            .map(Either::First)
            .or_else(|_| N::try_from(value).map(Either::Second))
            .map_err(|_| Rejection::new(value))
    }
}

impl<M, N, D> Stateful<D> for Either<M, N>
where
    M: Stateful<D>,
    N: Stateful<D>,
{
    #[allow(refining_impl_trait)]
    fn with(&self, data: &D) -> Response {
        match self {
            Self::First(media) => media.with(data).into_response(),
            Self::Second(media) => media.with(data).into_response(),
        }
    }
}

impl<T, M, N> payload::Extract<T> for Either<M, N>
where
    M: payload::Extract<T> + Send + Sync + 'static,
    N: payload::Extract<T> + Send + Sync + 'static,
{
    type Rejection = Either<M::Rejection, N::Rejection>;

    async fn extract(&self, req: Request) -> Result<T, Self::Rejection> {
        match self {
            Self::First(media) => {
                media.extract(req).map_err(Either::First).await
            }
            Self::Second(media) => {
                media.extract(req).map_err(Either::Second).await
            }
        }
    }
}

impl<M, N> Supported for Either<M, N>
where
    M: Supported,
    N: Supported,
{
    fn supported() -> Vec<Type> {
        let mut supported = M::supported();
        let mut other = N::supported();
        supported.append(&mut other);
        supported
    }
}

impl<M, N> IntoResponse for Either<M, N>
where
    M: IntoResponse,
    N: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Self::First(f) => f.into_response(),
            Self::Second(s) => s.into_response(),
        }
    }
}
