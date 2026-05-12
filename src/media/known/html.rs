//! Provides [Html] **media-type**.

use askama::Template;
use askama_web::WebTemplateExt;
use axum::response::{IntoResponse, Response};
use mime::TEXT_HTML;

use crate::media;

/// `text/html` **media-type**.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Html;

impl TryFrom<&media::Type> for Html {
    type Error = media::Rejection<Self>;

    fn try_from(value: &media::Type) -> Result<Self, Self::Error> {
        if value == &TEXT_HTML {
            Ok(Self)
        } else if value == &media::Type::default() {
            Ok(Self)
        } else {
            Err(media::Rejection::new(value))
        }
    }
}

impl<D: Template> media::Stateful<D> for Html {
    fn with(&self, data: &D) -> impl IntoResponse {
        data.into_web_template()
    }
}

impl<M, N> media::Stateful<media::Either<M, N>> for Html
where
    M: Template,
    N: Template,
{
    #[allow(refining_impl_trait)]
    fn with(&self, data: &media::Either<M, N>) -> Response {
        match data {
            media::Either::First(m) => m.into_web_template().into_response(),
            media::Either::Second(n) => n.into_web_template().into_response(),
        }
    }
}

impl media::Supported for Html {
    fn supported() -> Vec<media::Type> {
        vec![TEXT_HTML.into()]
    }
}
