//! # URI value codec (RFC 5545 3.3.13)
//!
//! [`Codec`] for the URI value. RFC 5545 section 3.3.13 gives a URI no
//! escaping and no structure, so its `;` and `,` are part of the reference:
//! the whole value is read, and written back exactly as it is held.

use crate::{
    tree::{
        codec::{Codec, encode::verbatim_node, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::uri::IcalUri,
};

impl<'v> Codec<'v> for IcalUri<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalUri(node.decode_joined())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        verbatim_node(&self.0, escaper)
    }
}
