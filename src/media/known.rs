//! Provides *known* **media-types** e.g. [Html].

pub mod csv;
mod form;
mod html;
mod json;

pub use form::Form;
pub use html::Html;
pub use json::Json;
