//! # Date and time value codecs (RFC 5545 3.3.4, 3.3.5, 3.3.12)
//!
//! [`Codec`] for the DATE, DATE-TIME and TIME values, each kept as raw text.

use crate::{
    tree::{
        codec::{Codec, encode::scalar_node, mode::Escaper},
        value::IcalValueNode,
    },
    value::datetime::{IcalDate, IcalDateTime, IcalTime},
};

impl<'v> Codec<'v> for IcalDate<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalDate(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
impl<'v> Codec<'v> for IcalDateTime<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalDateTime(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
impl<'v> Codec<'v> for IcalTime<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalTime(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
