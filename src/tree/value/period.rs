//! # Period value codec (RFC 5545 3.3.9)
//!
//! [`Codec`] for the period value. A value whose comma is literal, not a list
//! separator, so the whole component is kept.

use crate::{
    tree::{
        codec::{Codec, encode::scalar_node, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::period::IcalPeriod,
};

impl<'v> Codec<'v> for IcalPeriod<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalPeriod(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
