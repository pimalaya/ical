//! # Duration value codec (RFC 5545 3.3.6)
//!
//! [`Codec`] for the duration value. A single scalar value.

use crate::{
    tree::{
        codec::{Codec, encode::IcalValueOut, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::duration::IcalDuration,
};

impl<'v> Codec<'v> for IcalDuration<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalDuration(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalDuration<'_> {
    /// Write the duration as one component holding one value.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.component(&[&*self.0]);
    }
}
