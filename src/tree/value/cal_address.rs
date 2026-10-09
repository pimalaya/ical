//! # Calendar user address value codec (RFC 5545 3.3.3)
//!
//! [`Codec`] for the calendar user address value. A calendar user address is a
//! URI, so it reads and writes as one: the whole value is read, its `;` and `,`
//! kept as part of the address, and written back exactly as it is held, with
//! no text escaping (RFC 5545 3.3.13).

use crate::{
    tree::{
        codec::{Codec, encode::verbatim_node, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::cal_address::IcalCalAddress,
};

impl<'v> Codec<'v> for IcalCalAddress<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalCalAddress(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        verbatim_node(self.0.as_bytes(), escaper)
    }
}
