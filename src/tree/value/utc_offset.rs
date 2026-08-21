//! # UTC offset value codec (RFC 5545 3.3.14)
//!
//! [`Codec`] for the UTC offset value. A single scalar value.

use crate::{
    tree::{
        codec::{Codec, encode::scalar_node, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::utc_offset::IcalUtcOffset,
};

impl<'v> Codec<'v> for IcalUtcOffset<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalUtcOffset(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
