//! # calendar user address value codec (RFC 5545 3.3.3)
//!
//! [`Codec`] for the calendar user address value. A value whose comma is literal, not a list separator, so the whole component is kept.

use crate::{
    tree::{
        codec::{Codec, encode::scalar_node, mode::Escaper},
        value::IcalValueNode,
    },
    value::cal_address::IcalCalAddress,
};

impl<'v> Codec<'v> for IcalCalAddress<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalCalAddress(node.decode_joined_at(0))
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
