//! # Boolean value codec (RFC 5545 3.3.2)
//!
//! [`Codec`] for the boolean value. A single scalar value.

use crate::{
    tree::{
        codec::{Codec, encode::scalar_node, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::boolean::IcalBoolean,
};

impl<'v> Codec<'v> for IcalBoolean<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalBoolean(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
