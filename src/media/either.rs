//! Provides [Either] **media-type**.

use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use futures::TryFutureExt;

use crate::{Rejection, media, payload};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Either<M, N> {
    First(M),
    Second(N),
}

impl<'a, M, N> TryFrom<&'a media::Type> for Either<M, N>
where
    M: TryFrom<&'a media::Type> + media::Supported,
    N: TryFrom<&'a media::Type> + media::Supported,
{
    type Error = Rejection<Self>;

    fn try_from(value: &'a media::Type) -> Result<Self, Self::Error> {
        M::try_from(value)
            .map(Either::First)
            .or_else(|_| N::try_from(value).map(Either::Second))
            .map_err(|_| Rejection::new(value))
    }
}

impl<M, N, D> media::Stateful<D> for Either<M, N>
where
    M: media::Stateful<D>,
    N: media::Stateful<D>,
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

impl<M, N> media::Supported for Either<M, N>
where
    M: media::Supported,
    N: media::Supported,
{
    fn supported() -> Vec<media::Type> {
        let mut supported = M::supported();
        let mut other = N::supported();
        supported.append(&mut other);
        supported
    }
}
