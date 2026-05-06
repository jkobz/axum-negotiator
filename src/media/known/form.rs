//! Provides [Form] **media-type**.

use mime::{APPLICATION_JSON, APPLICATION_WWW_FORM_URLENCODED};

use crate::{Rejection, media};

/// **Media-types** associated with *form* submission.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Form {
    /// `application/x-www-form-urlencoded`
    #[default]
    Urlencoded,
    /// `application/json`
    Json,
}

impl TryFrom<media::Type> for Form {
    type Error = Rejection<Self>;

    fn try_from(value: media::Type) -> Result<Self, Self::Error> {
        if value == APPLICATION_JSON {
            Ok(Self::Json)
        } else if value == APPLICATION_WWW_FORM_URLENCODED {
            Ok(Self::Urlencoded)
        } else {
            Err(Rejection::new(value))
        }
    }
}

impl media::Supported for Form {
    fn supported() -> Vec<media::Type> {
        vec![
            APPLICATION_JSON.into(),
            APPLICATION_WWW_FORM_URLENCODED.into(),
        ]
    }
}
