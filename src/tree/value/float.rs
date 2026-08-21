//! # Float value codec (RFC 5545 3.3.7)
//!
//! [`Codec`] for the float value. A single scalar value.

use crate::{
    tree::{
        codec::{Codec, encode::scalar_node, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::float::IcalFloat,
};

impl<'v> Codec<'v> for IcalFloat<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalFloat(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
