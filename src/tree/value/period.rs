//! # Period value codec (RFC 5545 3.3.9)
//!
//! [`Codec`] for the period value. A value whose comma is literal, not a list
//! separator, so the whole component is kept.

use crate::{
    tree::{
        codec::{Codec, encode::IcalValueOut, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::period::IcalPeriod,
};

impl<'v> Codec<'v> for IcalPeriod<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalPeriod(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalPeriod<'_> {
    /// Write the period as one component holding one value.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.component(&[&*self.0]);
    }
}
