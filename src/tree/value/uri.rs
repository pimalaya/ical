//! # URI value codec (RFC 5545 3.3.13)
//!
//! [`Codec`] for the URI value. RFC 5545 section 3.3.13 gives a URI no
//! escaping and no structure, so its `;` and `,` are part of the reference:
//! the whole value is read, and written back exactly as it is held.

use crate::{
    tree::{
        codec::{Codec, encode::IcalValueOut, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::uri::IcalUri,
};

impl<'v> Codec<'v> for IcalUri<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalUri(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalUri<'_> {
    /// Write the URI exactly as it is held.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.uri(&self.0);
    }
}
