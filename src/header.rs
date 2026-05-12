//! Provides [Status] trait, and [Accept] and [ContentType] **HTTP** headers.

mod accept;
mod content_type;

pub use accept::Accept;
use axum::http::StatusCode;
pub use content_type::ContentType;

/// Provides `status` to populate [StatusCode].
pub trait Status {
    fn status(&self) -> StatusCode;
}
