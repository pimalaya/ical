//! # duration value codec (RFC 5545 3.3.6)
//!
//! [`Codec`] for the duration value. A single scalar value.

use crate::{
    tree::{
        codec::{Codec, encode::scalar_node, mode::Escaper},
        value::IcalValueNode,
    },
    value::duration::IcalDuration,
};

impl<'v> Codec<'v> for IcalDuration<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalDuration(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
