//! Provides **media-type** conversion [Rejection].

use std::marker::PhantomData;

use super::{Supported, Type};

/// **Media-type** conversion error.
pub struct Rejection<M: Supported> {
    /// *Rejected* **media-type**.
    pub media: Type,
    /// *Supported* **media-types** by an `M` media.
    supported_media: PhantomData<M>,
}

impl<M: Supported> Rejection<M> {
    pub fn new(media: &Type) -> Self {
        Self {
            media: media.clone(),
            supported_media: PhantomData,
        }
    }
}
