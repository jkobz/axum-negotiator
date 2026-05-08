//! Provides [Status] trait, and [Accept] **HTTP** request header.

mod accept;
mod content_type;

pub use accept::Accept;
pub use content_type::ContentType;

/// Provides `status` to populate [http::StatusCode].
pub trait Status {
    fn status(&self) -> axum::http::StatusCode;
}
