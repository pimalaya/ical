//! # Date and time value codecs (RFC 5545 3.3.4, 3.3.5, 3.3.12)
//!
//! [`Codec`] for the DATE, DATE-TIME and TIME values, each kept as raw text.

use alloc::vec;

use crate::{
    tree::{
        codec::{
            Codec,
            encode::{encode_component, scalar_node},
            mode::Escaper,
        },
        value::node::IcalValueNode,
    },
    value::datetime::{IcalDate, IcalDateTime, IcalDateTimeList, IcalTime},
};

impl<'v> Codec<'v> for IcalDate<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalDate(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
impl<'v> Codec<'v> for IcalDateTimeList<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalDateTimeList(node.decode_list())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueNode::from_components(vec![encode_component(&self.0, escaper)], escaper)
    }
}

impl<'v> Codec<'v> for IcalDateTime<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalDateTime(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
impl<'v> Codec<'v> for IcalTime<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalTime(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
