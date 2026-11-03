//! # UTC offset value codec (RFC 5545 3.3.14)
//!
//! [`Codec`] for the UTC offset value. A single scalar value.

use crate::{
    tree::{
        codec::{Codec, encode::IcalValueOut, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::utc_offset::IcalUtcOffset,
};

impl<'v> Codec<'v> for IcalUtcOffset<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalUtcOffset(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalUtcOffset<'_> {
    /// Write the offset as one component holding one value.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.component(&[&*self.0]);
    }
}
