//! # URI value codec (RFC 5545 3.3.13)
//!
//! [`Codec`] for the URI value. A value whose comma is literal, not a list
//! separator, so the whole component is kept.

use crate::{
    tree::{
        codec::{Codec, encode::scalar_node, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::uri::IcalUri,
};

impl<'v> Codec<'v> for IcalUri<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalUri(node.decode_joined_at(0))
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
