//! Provides [Form] **media-type**.

use axum::RequestExt;
use axum::extract::Request;
use futures::TryFutureExt;
use mime::APPLICATION_WWW_FORM_URLENCODED;
use serde::Deserialize;

use crate::{media, payload};

/// `application/x-www-form-urlencoded` **media-type**.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Form;

impl TryFrom<&media::Type> for Form {
    type Error = media::Rejection<Self>;

    fn try_from(value: &media::Type) -> Result<Self, Self::Error> {
        if value == &APPLICATION_WWW_FORM_URLENCODED {
            Ok(Self)
        } else {
            Err(media::Rejection::new(value))
        }
    }
}

impl<T> payload::Extract<T> for Form
where
    T: for<'a> Deserialize<'a> + 'static,
{
    type Rejection = axum_extra::extract::FormRejection;

    async fn extract(&self, req: Request) -> Result<T, Self::Rejection> {
        req.extract::<axum_extra::extract::Form<T>, _>()
            .map_ok(|f| f.0)
            .await
    }
}

impl media::Supported for Form {
    fn supported() -> Vec<media::Type> {
        vec![APPLICATION_WWW_FORM_URLENCODED.into()]
    }
}
