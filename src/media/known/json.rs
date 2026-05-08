//! Provides [Json] **media-type**.

use axum::RequestExt;
use axum::extract::Request;
use axum::response::IntoResponse;
use futures::TryFutureExt;
use mime::APPLICATION_JSON;
use serde::{Deserialize, Serialize};

use crate::{Rejection, media, payload};

/// `application/json` media-type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Json;

impl TryFrom<&media::Type> for Json {
    type Error = Rejection<Self>;

    fn try_from(value: &media::Type) -> Result<Self, Self::Error> {
        if value == &APPLICATION_JSON {
            Ok(Self)
        } else {
            Err(Rejection::new(value))
        }
    }
}

impl<D: Serialize> media::Stateful<D> for Json {
    fn with(&self, data: &D) -> impl IntoResponse {
        axum::extract::Json(data)
    }
}

impl<T> payload::Extract<T> for Json
where
    T: for<'a> Deserialize<'a> + 'static,
{
    type Rejection = axum::extract::rejection::JsonRejection;

    async fn extract(&self, req: Request) -> Result<T, Self::Rejection> {
        req.extract::<axum::extract::Json<T>, _>()
            .map_ok(|f| f.0)
            .await
    }
}

impl media::Supported for Json {
    fn supported() -> Vec<media::Type> {
        vec![APPLICATION_JSON.into()]
    }
}
