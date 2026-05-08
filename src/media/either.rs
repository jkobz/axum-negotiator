//! Provides [Either] **media-type**.

use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use futures::TryFutureExt;

use super::{Rejection, Stateful, Supported, Type};
use crate::payload;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Either<M, N> {
    First(M),
    Second(N),
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
    M: payload::Extract<T> + Send + Sync,
    N: payload::Extract<T> + Send + Sync,
{
    type Rejection = axum_extra::either::Either<
        <M as payload::Extract<T>>::Rejection,
        <N as payload::Extract<T>>::Rejection,
    >;

    async fn extract(&self, req: Request) -> Result<T, Self::Rejection> {
        match self {
            Self::First(media) => {
                media
                    .extract(req)
                    .map_err(axum_extra::either::Either::E1)
                    .await
            }
            Self::Second(media) => {
                media
                    .extract(req)
                    .map_err(axum_extra::either::Either::E2)
                    .await
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
