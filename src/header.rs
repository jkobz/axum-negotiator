//! Provides [Status] trait, and [Accept] **HTTP** request header.

mod accept;

pub use accept::Accept;

/// Provides `status` to populate [http::StatusCode].
pub trait Status {
    fn status(&self) -> http::StatusCode;
}
