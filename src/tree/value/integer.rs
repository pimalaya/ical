//! # Integer value codec (RFC 5545 3.3.8)
//!
//! [`Codec`] for the integer value. A single scalar value.

use crate::{
    tree::{
        codec::{Codec, encode::IcalValueOut, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::integer::IcalInteger,
};

impl<'v> Codec<'v> for IcalInteger<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalInteger(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalInteger<'_> {
    /// Write the integer as one component holding one value.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.component(&[&*self.0]);
    }
}
