//! # Boolean value codec (RFC 5545 3.3.2)
//!
//! [`Codec`] for the boolean value. A single scalar value.

use crate::{
    tree::{
        codec::{Codec, encode::IcalValueOut, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::boolean::IcalBoolean,
};

impl<'v> Codec<'v> for IcalBoolean<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalBoolean(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalBoolean<'_> {
    /// Write the boolean as one component holding one value.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.component(&[&*self.0]);
    }
}
