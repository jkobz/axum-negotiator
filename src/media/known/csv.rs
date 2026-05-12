//! Provides [Csv] **media-type**, [Named] trait.

use axum::http::header::{
    CONTENT_DISPOSITION, CONTENT_TYPE, HeaderMap, HeaderValue,
};
use axum::response::IntoResponse;
use mime::{TEXT_CSV, TEXT_HTML};

use super::Html;
use crate::media;

/// `text/csv` **media-type**.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Csv;

/// Provides [Named::filename] method.
pub trait Named {
    /// **CSV** filename.
    fn filename(&self) -> &str;
}

impl TryFrom<&media::Type> for Csv {
    type Error = media::Rejection<Self>;

    fn try_from(value: &media::Type) -> Result<Self, Self::Error> {
        if value == &TEXT_CSV {
            Ok(Self)
        } else if value == &TEXT_HTML {
            Ok(Self)
        } else if value == &media::Type::default() {
            Ok(Self)
        } else {
            Err(media::Rejection::new(value))
        }
    }
}

impl<D, E> media::Stateful<D> for Csv
where
    D: Named,
    Vec<u8>: for<'a> TryFrom<&'a D, Error = E>,
    Html: media::Stateful<E>,
{
    fn with(&self, data: &D) -> impl IntoResponse {
        let filename = data.filename().to_owned();
        Vec::<u8>::try_from(data).map_or_else(
            |err| Html.with(&err).into_response(),
            |bytes| {
                (
                    HeaderMap::from_iter([
                        (
                            CONTENT_TYPE,
                            HeaderValue::from_static("text/csv; charset=utf-8"),
                        ),
                        (
                            CONTENT_DISPOSITION,
                            HeaderValue::from_str(&format!(
                                r#"attachment; filename="{filename}.csv""#,
                            ))
                            .unwrap_or(
                                HeaderValue::from_static(
                                    r#"attachment; filename="export.csv""#,
                                ),
                            ),
                        ),
                    ]),
                    bytes,
                )
                    .into_response()
            },
        )
    }
}

impl media::Supported for Csv {
    fn supported() -> Vec<media::Type> {
        vec![TEXT_CSV.into(), TEXT_HTML.into()]
    }
}
