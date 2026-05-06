//! Provides content-negotiation [Rejection].

use std::marker::PhantomData;

use super::media;

pub struct Rejection<M>
where
    M: media::Supported,
{
    pub media: media::Type,
    supported_media: PhantomData<M>,
}

impl<M> Rejection<M>
where
    M: media::Supported,
{
    pub fn new(media: media::Type) -> Self {
        Self {
            media,
            supported_media: PhantomData,
        }
    }
}
