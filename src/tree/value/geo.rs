//! # GEO value codec (RFC 5545 3.8.1.6)
//!
//! [`Codec`] for the structured `GEO` value: a `latitude;longitude` pair of
//! FLOATs, held as two `;`-separated components.

use alloc::vec;

use crate::{
    tree::{
        codec::{Codec, encode::encode_component, mode::Escaper},
        value::IcalValueNode,
    },
    value::geo::IcalGeo,
};

impl<'v> Codec<'v> for IcalGeo<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalGeo {
            latitude: node.decode_scalar_at(0),
            longitude: node.decode_scalar_at(1),
        }
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueNode::from_components(
            vec![
                encode_component(&[self.latitude.as_ref()], escaper),
                encode_component(&[self.longitude.as_ref()], escaper),
            ],
            escaper,
        )
    }
}
