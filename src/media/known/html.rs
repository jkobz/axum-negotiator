//! Provides [Html] **media-type**.

use askama::Template;
use askama_web::WebTemplateExt;
use axum::response::IntoResponse;
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

impl media::Supported for Html {
    fn supported() -> Vec<media::Type> {
        vec![TEXT_HTML.into()]
    }
}
