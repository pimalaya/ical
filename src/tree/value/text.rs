//! # Text value codec (RFC 5545 3.3.11)
//!
//! [`Codec`] for a single text value and a comma-separated text list.

use crate::{
    tree::{
        codec::{Codec, encode::IcalValueOut, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::text::{IcalText, IcalTextList},
};

impl<'v> Codec<'v> for IcalText<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        // NOTE: A single text value has no list to be separated into, so an
        // unescaped comma in one is data rather than a separator. RFC 5545
        // 3.3.11 says it should have been escaped; truncating the value at it
        // would be strictness applied to the wrong end of Postel's law.
        IcalText(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalText<'_> {
    /// Write the text as one component holding one value.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.component(&[&*self.0]);
    }
}

impl<'v> Codec<'v> for IcalTextList<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalTextList(node.decode_list())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalTextList<'_> {
    /// Write the list as one component of its values.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.component(&self.0);
    }
}
