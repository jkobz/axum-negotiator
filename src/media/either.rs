//! Provides [Either] *opaque* **type**.

use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use futures::TryFutureExt;

use super::{Rejection, Stateful, Supported, Type};
use crate::{Negotiate, payload};

/// **Either-this-or-that** container.
///
/// Encapsulates both `A` and `B` types into a single
/// container type. Designed to work with **media-types** in mind,
/// however, can be used to work with any combination of types,
/// given they satisfy certain trait bounds:
///
/// - [Supported] and [Stateful] trait for **media-type** usage
/// - [IntoResponse] trait for extractor **rejection** usage
/// - [Negotiate] for **content-negotiation** usage
/// - [payload::Extract] for **payload-extraction** usage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Either<A, B> {
    First(A),
    Second(B),
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

impl<A: Default, B> Default for Either<A, B> {
    fn default() -> Self {
        Either::First(A::default())
    }
}

impl<'a, A, B> TryFrom<&'a Type> for Either<A, B>
where
    A: TryFrom<&'a Type> + Supported,
    B: TryFrom<&'a Type> + Supported,
{
    type Error = Rejection<Self>;

    fn try_from(value: &'a Type) -> Result<Self, Self::Error> {
        A::try_from(value)
            .map(Either::First)
            .or_else(|_| B::try_from(value).map(Either::Second))
            .map_err(|_| Rejection::new(value))
    }
}

impl<A, B, D> Stateful<D> for Either<A, B>
where
    A: Stateful<D>,
    B: Stateful<D>,
{
    #[allow(refining_impl_trait)]
    fn with(&self, data: &D) -> Response {
        match self {
            Self::First(media) => media.with(data).into_response(),
            Self::Second(media) => media.with(data).into_response(),
        }
    }
}

impl<T, A, B> payload::Extract<T> for Either<A, B>
where
    A: payload::Extract<T> + Send + Sync + 'static,
    B: payload::Extract<T> + Send + Sync + 'static,
{
    type Rejection = Either<A::Rejection, B::Rejection>;

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

impl<A, B> Supported for Either<A, B>
where
    A: Supported,
    B: Supported,
{
    fn supported() -> Vec<Type> {
        let mut supported = A::supported();
        let mut other = B::supported();
        supported.append(&mut other);
        supported
    }
}

impl<A, B> IntoResponse for Either<A, B>
where
    A: IntoResponse,
    B: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Self::First(f) => f.into_response(),
            Self::Second(s) => s.into_response(),
        }
    }
}
