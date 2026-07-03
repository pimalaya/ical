//! # Text value codec (RFC 5545 3.3.11)
//!
//! [`Codec`] for a single text value and a comma-separated text list.

use alloc::vec;

use crate::{
    tree::{
        codec::{
            Codec,
            encode::{encode_component, scalar_node},
            mode::Escaper,
        },
        value::IcalValueNode,
    },
    value::text::{IcalText, IcalTextList},
};

impl<'v> Codec<'v> for IcalText<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalText(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}

impl<'v> Codec<'v> for IcalTextList<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalTextList(node.decode_at(0))
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueNode::from_components(vec![encode_component(&self.0, escaper)], escaper)
    }
}
