//! # Float value codec (RFC 5545 3.3.7)
//!
//! [`Codec`] for the float value. A single scalar value.

use crate::{
    tree::{
        codec::{Codec, encode::IcalValueOut, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::float::IcalFloat,
};

impl<'v> Codec<'v> for IcalFloat<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalFloat(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalFloat<'_> {
    /// Write the float as one component holding one value.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.component(&[&*self.0]);
    }
}
