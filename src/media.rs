//! Provides **media-type** via [Type].

mod either;
mod error;
pub mod extract;
mod known;

use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use axum::response::IntoResponse;
pub use either::Either;
pub use error::Rejection;
pub use extract::Extractor;
pub use known::csv::{self, Csv};
pub use known::{Form, Html, Json};
use mime::{Mime, STAR_STAR};

/// Turns an arbitrary `data` into an [IntoResponse]
/// implementation via `with` method.
pub trait Stateful<D> {
    fn with(&self, data: &D) -> impl IntoResponse;
}

/// Provides *supported* **media-type** list via `supported` method.
pub trait Supported {
    fn supported() -> Vec<Type>;
}

/// # **HTTP** **MIME** media-type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type(pub Mime);

impl Default for Type {
    fn default() -> Self {
        Self(STAR_STAR)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for Type {
    type Target = Mime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&Mime> for Type {
    fn from(value: &Mime) -> Self {
        Self(value.clone())
    }
}

impl From<Mime> for Type {
    fn from(value: Mime) -> Self {
        Self(value)
    }
}

impl Into<Mime> for Type {
    fn into(self) -> Mime {
        self.0
    }
}

impl PartialEq<Mime> for Type {
    fn eq(&self, other: &Mime) -> bool {
        &self.0 == other
    }
}

impl PartialEq<Type> for Mime {
    fn eq(&self, other: &Type) -> bool {
        self == &other.0
    }
}

impl FromStr for Type {
    type Err = mime::FromStrError;

    fn from_str(media: &str) -> Result<Self, Self::Err> {
        Mime::from_str(media).map(Self)
    }
}
