//! # Binary value codec (RFC 5545 3.3.1)
//!
//! [`Codec`] for an `ATTACH` / `IMAGE` value: inline
//! base64 kept verbatim. A URI reference is reached via `VALUE=uri`, which the
//! spec resolves to [`IcalUri`](crate::value::uri::IcalUri), not here.

use crate::{
    tree::{
        codec::{Codec, encode::scalar_node, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::binary::IcalBinary,
};

impl<'v> Codec<'v> for IcalBinary<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalBinary::Base64(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        let raw = match self {
            IcalBinary::Uri(value) | IcalBinary::Base64(value) => value.as_ref(),
        };

        scalar_node(raw, escaper)
    }
}
