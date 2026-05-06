//! Provides [Html] **media-type**.

use askama::Template;
use askama_web::WebTemplateExt;
use axum_core::response::IntoResponse;
use mime::TEXT_HTML;

use crate::{Rejection, media};

/// `text/html` **media-type**.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Html;

impl TryFrom<media::Type> for Html {
    type Error = Rejection<Self>;

    fn try_from(value: media::Type) -> Result<Self, Self::Error> {
        if value == TEXT_HTML {
            Ok(Self)
        } else if value == media::Type::default() {
            Ok(Self)
        } else {
            Err(Rejection::new(value))
        }
    }
}

impl<D> media::Stateful<D> for Html
where
    D: Template,
{
    fn with(&self, data: &D) -> impl IntoResponse {
        data.into_web_template().into_response()
    }
}

impl media::Supported for Html {
    fn supported() -> Vec<media::Type> {
        vec![mime::TEXT_HTML.into()]
    }
}
