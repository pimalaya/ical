//! # Integer value codec (RFC 5545 3.3.8)
//!
//! [`Codec`] for the integer value. A single scalar value.

use crate::{
    tree::{
        codec::{Codec, encode::scalar_node, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::integer::IcalInteger,
};

impl<'v> Codec<'v> for IcalInteger<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalInteger(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
